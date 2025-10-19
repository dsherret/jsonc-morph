use js_sys::JsString;
use jsonc_parser::ParseOptions;
use jsonc_parser::cst;
use jsonc_parser::cst::CstContainerNode;
use jsonc_parser::cst::CstInputValue;
use jsonc_parser::cst::CstLeafNode;
use jsonc_parser::cst::CstNode as JsoncCstNode;
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn throw_error(msg: &str) -> JsValue {
  js_sys::Error::new(msg).into()
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(
    typescript_type = "{ allowComments?: boolean; allowTrailingCommas?: boolean; allowLooseObjectPropertyNames?: boolean; }"
  )]
  pub type JsoncParseOptionsObject;

  #[wasm_bindgen(
    typescript_type = "string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue }"
  )]
  pub type JsonValue;
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };
"#;

/// Parses a JSONC (JSON with Comments) string into a concrete syntax tree.
/// @param text - The JSONC text to parse
/// @param options - Optional parsing options
/// @returns The root node of the parsed CST
#[wasm_bindgen]
pub fn parse(
  text: &str,
  options: Option<JsoncParseOptionsObject>,
) -> Result<RootNode, JsValue> {
  let parse_options = match options {
    Some(opts) => parse_options_from_js(&opts.into()),
    None => ParseOptions::default(),
  };

  let root = cst::CstRootNode::parse(text, &parse_options)
    .map_err(|e| throw_error(&format!("Parse error: {}", e.kind())))?;
  Ok(RootNode { inner: root })
}

/// Parses a JSONC (JSON with Comments) string directly to a JavaScript object.
/// @param text - The JSONC text to parse
/// @param options - Optional parsing options
/// @returns The plain JavaScript value (object, array, string, number, boolean, or null)
/// @throws If the text cannot be parsed or converted
#[wasm_bindgen(js_name = parseToValue)]
pub fn parse_to_value(
  text: &str,
  options: Option<JsoncParseOptionsObject>,
) -> Result<JsValue, JsValue> {
  let parse_options = match options {
    Some(opts) => parse_options_from_js(&opts.into()),
    None => ParseOptions::default(),
  };

  // Use the more efficient parse_to_serde_value API from jsonc_parser
  // This skips building the full CST and directly produces a serde_json::Value
  let serde_value = jsonc_parser::parse_to_serde_value(text, &parse_options)
    .map_err(|e| throw_error(&format!("Parse error: {}", e)))?;

  // Convert serde_json::Value to JsValue using serde-wasm-bindgen with custom serializer
  // Use serialize_maps_as_objects to get plain JS objects instead of Maps
  let serializer = serde_wasm_bindgen::Serializer::json_compatible();
  serde_value.serialize(&serializer)
    .map_err(|e| throw_error(&format!("Failed to convert value: {}", e)))
}

fn parse_options_from_js(obj: &JsValue) -> ParseOptions {
  let defaults = ParseOptions::default();

  if !obj.is_object() {
    return defaults;
  }

  let allow_comments = js_sys::Reflect::get(obj, &"allowComments".into())
    .ok()
    .and_then(|v| v.as_bool())
    .unwrap_or(defaults.allow_comments);

  let allow_trailing_commas =
    js_sys::Reflect::get(obj, &"allowTrailingCommas".into())
      .ok()
      .and_then(|v| v.as_bool())
      .unwrap_or(defaults.allow_trailing_commas);

  let allow_loose_object_property_names =
    js_sys::Reflect::get(obj, &"allowLooseObjectPropertyNames".into())
      .ok()
      .and_then(|v| v.as_bool())
      .unwrap_or(defaults.allow_loose_object_property_names);

  ParseOptions {
    allow_comments,
    allow_trailing_commas,
    allow_loose_object_property_names,
  }
}

fn js_value_to_cst_input(value: &JsValue) -> Result<CstInputValue, JsValue> {
  // Convert JsValue to serde_json::Value using serde-wasm-bindgen
  let serde_value: serde_json::Value =
    serde_wasm_bindgen::from_value(value.clone())
      .map_err(|e| throw_error(&format!("Failed to convert value: {}", e)))?;

  // Convert serde_json::Value to CstInputValue
  Ok(convert_serde_to_cst_input(serde_value))
}

fn convert_serde_to_cst_input(value: serde_json::Value) -> CstInputValue {
  match value {
    serde_json::Value::Null => CstInputValue::Null,
    serde_json::Value::Bool(b) => CstInputValue::from(b),
    serde_json::Value::Number(n) => {
      if let Some(i) = n.as_i64() {
        CstInputValue::from(i)
      } else if let Some(f) = n.as_f64() {
        CstInputValue::from(f)
      } else {
        CstInputValue::Null
      }
    }
    serde_json::Value::String(s) => CstInputValue::from(s),
    serde_json::Value::Array(arr) => {
      let converted: Vec<CstInputValue> =
        arr.into_iter().map(convert_serde_to_cst_input).collect();
      CstInputValue::from(converted)
    }
    serde_json::Value::Object(obj) => {
      let converted: Vec<(String, CstInputValue)> = obj
        .into_iter()
        .map(|(k, v)| (k, convert_serde_to_cst_input(v)))
        .collect();
      CstInputValue::from(converted)
    }
  }
}

thread_local! {
  static LF: JsString = JsString::from("\n");
  static CRLF: JsString = JsString::from("\r\n");
}

/// Represents the root node of a JSONC document.
/// This is the entry point for manipulating the concrete syntax tree.
#[wasm_bindgen]
pub struct RootNode {
  inner: cst::CstRootNode,
}

#[wasm_bindgen]
impl RootNode {
  /// Returns the root value node.
  /// @returns The root value, or undefined if the document is empty
  #[wasm_bindgen(js_name = value)]
  pub fn value(&self) -> Option<Node> {
    self.inner.value().map(|v| Node { inner: v })
  }

  /// Returns the root value node, throwing if empty.
  /// @returns The root value
  /// @throws If the document is empty
  #[wasm_bindgen(js_name = valueOrThrow)]
  pub fn value_or_throw(&self) -> Result<Node, JsValue> {
    self
      .value()
      .ok_or_else(|| throw_error("Expected a value, but found none"))
  }

  /// Returns the root value as an object if it is one.
  /// @returns The object, or undefined if root is not an object
  #[wasm_bindgen(js_name = asObject)]
  pub fn as_object(&self) -> Option<JsonObject> {
    self.inner.object_value().map(|o| JsonObject { inner: o })
  }

  /// Returns the root value as an object, throwing if it's not an object.
  /// @returns The object
  /// @throws If the root is not an object
  #[wasm_bindgen(js_name = asObjectOrThrow)]
  pub fn as_object_or_throw(&self) -> Result<JsonObject, JsValue> {
    self.as_object().ok_or_else(|| {
      throw_error("Expected an object value, but found a different type")
    })
  }

  /// Returns the root value as an object, creating an empty object if the root is empty.
  /// Returns undefined if the root contains a value of a different type.
  /// @returns The object, or undefined if a non-object value exists
  #[wasm_bindgen(js_name = asObjectOrCreate)]
  pub fn as_object_or_create(&self) -> Option<JsonObject> {
    self
      .inner
      .object_value_or_create()
      .map(|o| JsonObject { inner: o })
  }

  /// Returns the root value as an object, replacing any existing value with an empty object if needed.
  /// Unlike asObjectOrCreate, this always returns an object by replacing non-object values.
  /// @returns The object (always succeeds)
  #[wasm_bindgen(js_name = asObjectOrForce)]
  pub fn as_object_or_force(&self) -> JsonObject {
    JsonObject {
      inner: self.inner.object_value_or_set(),
    }
  }

  /// Returns the root value as an array if it is one.
  /// @returns The array, or undefined if root is not an array
  #[wasm_bindgen(js_name = asArray)]
  pub fn as_array(&self) -> Option<JsonArray> {
    self.inner.array_value().map(|a| JsonArray { inner: a })
  }

  /// Returns the root value as an array, throwing if it's not an array.
  /// @returns The array
  /// @throws If the root is not an array
  #[wasm_bindgen(js_name = asArrayOrThrow)]
  pub fn as_array_or_throw(&self) -> Result<JsonArray, JsValue> {
    self.as_array().ok_or_else(|| {
      throw_error("Expected an array value, but found a different type")
    })
  }

  /// Returns the root value as an array, creating an empty array if the root is empty.
  /// Returns undefined if the root contains a value of a different type.
  /// @returns The array, or undefined if a non-array value exists
  #[wasm_bindgen(js_name = asArrayOrCreate)]
  pub fn as_array_or_create(&self) -> Option<JsonArray> {
    self
      .inner
      .array_value_or_create()
      .map(|a| JsonArray { inner: a })
  }

  /// Returns the root value as an array, replacing any existing value with an empty array if needed.
  /// Unlike asArrayOrCreate, this always returns an array by replacing non-array values.
  /// @returns The array (always succeeds)
  #[wasm_bindgen(js_name = asArrayOrForce)]
  pub fn as_array_or_force(&self) -> JsonArray {
    JsonArray {
      inner: self.inner.array_value_or_set(),
    }
  }

  /// Converts the CST back to a string representation.
  /// @returns The JSONC string
  #[wasm_bindgen(js_name = toString)]
  pub fn to_string_output(&self) -> String {
    self.inner.to_string()
  }

  /// Returns all child nodes including whitespace and punctuation.
  /// @returns Array of all child nodes
  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Sets the root value of the document.
  /// Accepts any JSON value: string, number, boolean, null, array, or object.
  /// @param value - The new value to set
  #[wasm_bindgen(js_name = setValue)]
  pub fn set_value(&self, value: JsValue) -> Result<(), JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    self.inner.set_value(cst_input);
    Ok(())
  }

  /// Configures whether trailing commas should be used throughout the document.
  /// When enabled, trailing commas are added for multiline formatting in objects and arrays.
  /// @param enabled - Whether to enable trailing commas
  #[wasm_bindgen(js_name = setTrailingCommas)]
  pub fn set_trailing_commas(&self, enabled: bool) {
    use jsonc_parser::cst::TrailingCommaMode;
    let mode = if enabled {
      TrailingCommaMode::IfMultiline
    } else {
      TrailingCommaMode::Never
    };
    self.inner.set_trailing_commas(mode);
  }

  /// Clears all children from the root node, leaving an empty document.
  #[wasm_bindgen(js_name = clearChildren)]
  pub fn clear_children(&self) {
    self.inner.clear_children();
  }

  /// Returns the indentation string used for a single level.
  /// @returns The single-level indentation string (e.g., "  " or "\t")
  #[wasm_bindgen(js_name = singleIndentText)]
  pub fn single_indent_text(&self) -> Option<String> {
    self.inner.single_indent_text()
  }

  /// Returns the newline kind used in the document.
  /// @returns Either "\n" or "\r\n"
  #[wasm_bindgen(js_name = newlineKind)]
  pub fn newline_kind(&self) -> JsString {
    match self.inner.newline_kind() {
      cst::CstNewlineKind::LineFeed => LF.with(|s| s.clone()),
      cst::CstNewlineKind::CarriageReturnLineFeed => CRLF.with(|s| s.clone()),
    }
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  /// Returns child nodes excluding whitespace, comments, and punctuation.
  /// @returns Array of significant child nodes
  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Returns the child node at the specified index.
  /// @param index - The child index
  /// @returns The child node, or undefined if index is out of bounds
  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }

  /// Converts the CST to a plain JavaScript value, similar to JSON.parse.
  /// This recursively converts the root value to its JavaScript equivalent.
  /// Comments and formatting information are discarded.
  /// @returns The plain JavaScript value (object, array, string, number, boolean, or null)
  /// @throws If the document contains invalid values that cannot be converted
  #[wasm_bindgen(js_name = toValue)]
  pub fn to_value(&self) -> JsValue {
    if let Some(value_node) = self.value() {
      value_node.to_value()
    } else {
      JsValue::UNDEFINED
    }
  }
}

/// Represents a generic node in the CST.
/// Can be a container node (object, array, property) or a leaf node (string, number, boolean, null).
#[wasm_bindgen]
#[derive(Clone)]
pub struct Node {
  inner: JsoncCstNode,
}

#[wasm_bindgen]
impl Node {
  /// Returns true if this node is a container (object, array, or property).
  /// @returns true if this is a container node
  #[wasm_bindgen(js_name = isContainer)]
  pub fn is_container(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Container(_))
  }

  /// Returns true if this node is a leaf value (string, number, boolean, null).
  /// @returns true if this is a leaf node
  #[wasm_bindgen(js_name = isLeaf)]
  pub fn is_leaf(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(_))
  }

  /// Converts this node to an object if it is one.
  /// @returns The object, or undefined if this node is not an object
  #[wasm_bindgen(js_name = asObject)]
  pub fn as_object(&self) -> Option<JsonObject> {
    match &self.inner {
      JsoncCstNode::Container(CstContainerNode::Object(obj)) => {
        Some(JsonObject { inner: obj.clone() })
      }
      _ => None,
    }
  }

  /// Converts this node to an object, throwing if it's not an object.
  /// @returns The object
  /// @throws If this node is not an object
  #[wasm_bindgen(js_name = asObjectOrThrow)]
  pub fn as_object_or_throw(&self) -> Result<JsonObject, JsValue> {
    match &self.inner {
      JsoncCstNode::Container(CstContainerNode::Object(obj)) => {
        Ok(JsonObject { inner: obj.clone() })
      }
      _ => Err(throw_error(
        "Expected an object node, but found a different type",
      )),
    }
  }

  /// Converts this node to an array if it is one.
  /// @returns The array, or undefined if this node is not an array
  #[wasm_bindgen(js_name = asArray)]
  pub fn as_array(&self) -> Option<JsonArray> {
    match &self.inner {
      JsoncCstNode::Container(CstContainerNode::Array(arr)) => {
        Some(JsonArray { inner: arr.clone() })
      }
      _ => None,
    }
  }

  /// Converts this node to an array, throwing if it's not an array.
  /// @returns The array
  /// @throws If this node is not an array
  #[wasm_bindgen(js_name = asArrayOrThrow)]
  pub fn as_array_or_throw(&self) -> Result<JsonArray, JsValue> {
    match &self.inner {
      JsoncCstNode::Container(CstContainerNode::Array(arr)) => {
        Ok(JsonArray { inner: arr.clone() })
      }
      _ => Err(throw_error(
        "Expected an array node, but found a different type",
      )),
    }
  }

  /// Converts this node to the root node if it is one.
  /// @returns The root node, or undefined if this is not a root node
  #[wasm_bindgen(js_name = asRootNode)]
  pub fn as_root_node(&self) -> Option<RootNode> {
    self.inner.as_root_node().map(|r| RootNode { inner: r })
  }

  /// Converts this node to the root node, throwing if it's not a root node.
  /// @returns The root node
  /// @throws If this node is not a root node
  #[wasm_bindgen(js_name = asRootNodeOrThrow)]
  pub fn as_root_node_or_throw(&self) -> Result<RootNode, JsValue> {
    self.as_root_node().ok_or_else(|| {
      throw_error("Expected a root node, but found a different type")
    })
  }

  /// Returns the decoded string value if this node is a string literal.
  /// @returns The string value, or undefined if this node is not a string
  #[wasm_bindgen(js_name = asString)]
  pub fn as_string(&self) -> Option<String> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::StringLit(s)) => s.decoded_value().ok(),
      _ => None,
    }
  }

  /// Returns the decoded string value, throwing if not a string.
  /// @returns The string value
  /// @throws If this node is not a string
  #[wasm_bindgen(js_name = asStringOrThrow)]
  pub fn as_string_or_throw(&self) -> Result<String, JsValue> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::StringLit(s)) => s
        .decoded_value()
        .map_err(|e| throw_error(&format!("Failed to decode string: {}", e))),
      _ => Err(throw_error(
        "Expected a string node, but found a different type",
      )),
    }
  }

  /// Returns the raw string representation of a number literal.
  /// Returns a string to preserve the exact formatting (e.g., "1.0" vs "1", "1e10" vs "10000000000").
  /// @returns The number as a string, or undefined if this node is not a number
  #[wasm_bindgen(js_name = numberValue)]
  pub fn number_value(&self) -> Option<String> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::NumberLit(n)) => Some(n.to_string()),
      _ => None,
    }
  }

  /// Returns the raw string representation of a number literal, throwing if not a number.
  /// Returns a string to preserve the exact formatting (e.g., "1.0" vs "1", "1e10" vs "10000000000").
  /// @returns The number as a string
  /// @throws If this node is not a number
  #[wasm_bindgen(js_name = numberValueOrThrow)]
  pub fn number_value_or_throw(&self) -> Result<String, JsValue> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::NumberLit(n)) => Ok(n.to_string()),
      _ => Err(throw_error(
        "Expected a number node, but found a different type",
      )),
    }
  }

  /// Returns the boolean value if this node is a boolean literal.
  /// @returns The boolean value, or undefined if this node is not a boolean
  #[wasm_bindgen(js_name = asBoolean)]
  pub fn as_boolean(&self) -> Option<bool> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::BooleanLit(b)) => Some(b.value()),
      _ => None,
    }
  }

  /// Returns the boolean value, throwing if not a boolean.
  /// @returns The boolean value
  /// @throws If this node is not a boolean
  #[wasm_bindgen(js_name = asBooleanOrThrow)]
  pub fn as_boolean_or_throw(&self) -> Result<bool, JsValue> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::BooleanLit(b)) => Ok(b.value()),
      _ => Err(throw_error(
        "Expected a boolean node, but found a different type",
      )),
    }
  }

  /// Returns true if this node is a null keyword.
  /// @returns true if this node represents null
  #[wasm_bindgen(js_name = isNull)]
  pub fn is_null(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(CstLeafNode::NullKeyword(_)))
  }

  /// Returns true if this node is a string literal.
  /// @returns true if this node is a string
  #[wasm_bindgen(js_name = isString)]
  pub fn is_string(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(CstLeafNode::StringLit(_)))
  }

  /// Returns true if this node is a number literal.
  /// @returns true if this node is a number
  #[wasm_bindgen(js_name = isNumber)]
  pub fn is_number(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(CstLeafNode::NumberLit(_)))
  }

  /// Returns true if this node is a boolean literal.
  /// @returns true if this node is a boolean
  #[wasm_bindgen(js_name = isBoolean)]
  pub fn is_boolean(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(CstLeafNode::BooleanLit(_)))
  }

  /// Returns this node as a StringLit if it is one.
  /// @returns The StringLit, or undefined if this node is not a string literal
  #[wasm_bindgen(js_name = asStringLit)]
  pub fn as_string_lit(&self) -> Option<StringLit> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::StringLit(s)) => {
        Some(StringLit { inner: s.clone() })
      }
      _ => None,
    }
  }

  /// Returns this node as a StringLit, throwing if it's not a string literal.
  /// @returns The StringLit
  /// @throws If this node is not a string literal
  #[wasm_bindgen(js_name = asStringLitOrThrow)]
  pub fn as_string_lit_or_throw(&self) -> Result<StringLit, JsValue> {
    self.as_string_lit().ok_or_else(|| {
      throw_error("Expected a string literal node, but found a different type")
    })
  }

  /// Returns this node as a NumberLit if it is one.
  /// @returns The NumberLit, or undefined if this node is not a number literal
  #[wasm_bindgen(js_name = asNumberLit)]
  pub fn as_number_lit(&self) -> Option<NumberLit> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::NumberLit(n)) => {
        Some(NumberLit { inner: n.clone() })
      }
      _ => None,
    }
  }

  /// Returns this node as a NumberLit, throwing if it's not a number literal.
  /// @returns The NumberLit
  /// @throws If this node is not a number literal
  #[wasm_bindgen(js_name = asNumberLitOrThrow)]
  pub fn as_number_lit_or_throw(&self) -> Result<NumberLit, JsValue> {
    self.as_number_lit().ok_or_else(|| {
      throw_error("Expected a number literal node, but found a different type")
    })
  }

  /// Returns this node as a BooleanLit if it is one.
  /// @returns The BooleanLit, or undefined if this node is not a boolean literal
  #[wasm_bindgen(js_name = asBooleanLit)]
  pub fn as_boolean_lit(&self) -> Option<BooleanLit> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::BooleanLit(b)) => {
        Some(BooleanLit { inner: b.clone() })
      }
      _ => None,
    }
  }

  /// Returns this node as a BooleanLit, throwing if it's not a boolean literal.
  /// @returns The BooleanLit
  /// @throws If this node is not a boolean literal
  #[wasm_bindgen(js_name = asBooleanLitOrThrow)]
  pub fn as_boolean_lit_or_throw(&self) -> Result<BooleanLit, JsValue> {
    self.as_boolean_lit().ok_or_else(|| {
      throw_error("Expected a boolean literal node, but found a different type")
    })
  }

  /// Returns this node as a NullKeyword if it is one.
  /// @returns The NullKeyword, or undefined if this node is not a null keyword
  #[wasm_bindgen(js_name = asNullKeyword)]
  pub fn as_null_keyword(&self) -> Option<NullKeyword> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::NullKeyword(n)) => {
        Some(NullKeyword { inner: n.clone() })
      }
      _ => None,
    }
  }

  /// Returns this node as a NullKeyword, throwing if it's not a null keyword.
  /// @returns The NullKeyword
  /// @throws If this node is not a null keyword
  #[wasm_bindgen(js_name = asNullKeywordOrThrow)]
  pub fn as_null_keyword_or_throw(&self) -> Result<NullKeyword, JsValue> {
    self.as_null_keyword().ok_or_else(|| {
      throw_error("Expected a null keyword node, but found a different type")
    })
  }

  /// Returns this node as a WordLit if it is one.
  /// @returns The WordLit, or undefined if this node is not a word literal
  #[wasm_bindgen(js_name = asWordLit)]
  pub fn as_word_lit(&self) -> Option<WordLit> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::WordLit(w)) => {
        Some(WordLit { inner: w.clone() })
      }
      _ => None,
    }
  }

  /// Returns this node as a WordLit, throwing if it's not a word literal.
  /// @returns The WordLit
  /// @throws If this node is not a word literal
  #[wasm_bindgen(js_name = asWordLitOrThrow)]
  pub fn as_word_lit_or_throw(&self) -> Result<WordLit, JsValue> {
    self.as_word_lit().ok_or_else(|| {
      throw_error("Expected a word literal node, but found a different type")
    })
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns the parent node, throwing if this is the root.
  /// @returns The parent node
  /// @throws If this node has no parent
  #[wasm_bindgen(js_name = parentOrThrow)]
  pub fn parent_or_throw(&self) -> Result<Node, JsValue> {
    self
      .parent()
      .ok_or_else(|| throw_error("Expected a parent node, but found none"))
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the root node, throwing if detached.
  /// @returns The root node
  /// @throws If this node is detached from the CST
  #[wasm_bindgen(js_name = rootNodeOrThrow)]
  pub fn root_node_or_throw(&self) -> Result<RootNode, JsValue> {
    self
      .root_node()
      .ok_or_else(|| throw_error("Expected a root node, but found none"))
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  /// Returns true if this node is trivia (whitespace or comments).
  /// @returns true if this node is trivia
  #[wasm_bindgen(js_name = isTrivia)]
  pub fn is_trivia(&self) -> bool {
    self.inner.is_trivia()
  }

  /// Returns true if this node is a newline character.
  /// @returns true if this node is a newline
  #[wasm_bindgen(js_name = isNewline)]
  pub fn is_newline(&self) -> bool {
    self.inner.is_newline()
  }

  /// Returns true if this node is a comma token.
  /// @returns true if this node is a comma
  #[wasm_bindgen(js_name = isComma)]
  pub fn is_comma(&self) -> bool {
    self.inner.is_comma()
  }

  /// Returns true if this node is a comment.
  /// @returns true if this node is a comment
  #[wasm_bindgen(js_name = isComment)]
  pub fn is_comment(&self) -> bool {
    self.inner.is_comment()
  }

  /// Returns true if this node is a punctuation token (bracket, brace, colon, comma).
  /// @returns true if this node is a token
  #[wasm_bindgen(js_name = isToken)]
  pub fn is_token(&self) -> bool {
    self.inner.is_token()
  }

  /// Returns true if this node is whitespace.
  /// @returns true if this node is whitespace
  #[wasm_bindgen(js_name = isWhitespace)]
  pub fn is_whitespace(&self) -> bool {
    self.inner.is_whitespace()
  }

  /// Returns the character if this node is a single-character token.
  /// @returns The token character, or undefined if not a token
  #[wasm_bindgen(js_name = tokenChar)]
  pub fn token_char(&self) -> Option<String> {
    self.inner.token_char().map(|c| c.to_string())
  }

  /// Returns the element index if this node is an array element.
  /// @returns The element index, or undefined if not an array element
  #[wasm_bindgen(js_name = elementIndex)]
  pub fn element_index(&self) -> Option<usize> {
    self.inner.element_index()
  }

  /// Returns all child nodes including whitespace and punctuation.
  /// @returns Array of all child nodes
  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Returns child nodes excluding whitespace, comments, and punctuation.
  /// @returns Array of significant child nodes
  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Returns the child node at the specified index.
  /// @param index - The child index
  /// @returns The child node, or undefined if index is out of bounds
  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }

  /// Converts this CST node to a plain JavaScript value.
  /// This recursively converts objects, arrays, and primitives to their JavaScript equivalents.
  /// Comments and formatting information are discarded.
  /// @returns The plain JavaScript value (object, array, string, number, boolean, or null)
  /// @throws If the node contains invalid values that cannot be converted
  #[wasm_bindgen(js_name = toValue)]
  pub fn to_value(&self) -> JsValue {
    match self.inner.to_serde_value() {
      Some(value) => {
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        value.serialize(&serializer).unwrap_or(JsValue::UNDEFINED)
      }
      None => {
        JsValue::UNDEFINED
      }
    }
  } 
}

/// Represents a JSON object node in the CST.
/// Provides methods for manipulating object properties.
#[wasm_bindgen]
#[derive(Clone)]
pub struct JsonObject {
  inner: cst::CstObject,
}

#[wasm_bindgen]
impl JsonObject {
  /// Returns all properties in the object.
  /// @returns Array of object properties
  #[wasm_bindgen(js_name = properties)]
  pub fn properties(&self) -> Vec<ObjectProp> {
    self
      .inner
      .properties()
      .into_iter()
      .map(|p| ObjectProp { inner: p })
      .collect()
  }

  /// Gets a property by name.
  /// @param key - The property name to look up
  /// @returns The property, or undefined if not found
  #[wasm_bindgen(js_name = get)]
  pub fn get(&self, key: &str) -> Option<ObjectProp> {
    self.inner.get(key).map(|p| ObjectProp { inner: p })
  }

  /// Gets a property by name, throwing if not found.
  /// @param key - The property name to look up
  /// @returns The property
  /// @throws If the property is not found
  #[wasm_bindgen(js_name = getOrThrow)]
  pub fn get_or_throw(&self, key: &str) -> Result<ObjectProp, JsValue> {
    self.get(key).ok_or_else(|| {
      throw_error(&format!(
        "Expected property '{}', but it was not found",
        key
      ))
    })
  }

  /// Gets a property value if it's an object.
  /// @param name - The property name to look up
  /// @returns The object value, or undefined if property doesn't exist or is not an object
  #[wasm_bindgen(js_name = getIfObject)]
  pub fn get_if_object(&self, name: &str) -> Option<JsonObject> {
    self
      .inner
      .object_value(name)
      .map(|o| JsonObject { inner: o })
  }

  /// Gets a property value as an object, throwing if not found or wrong type.
  /// @param name - The property name to look up
  /// @returns The object value
  /// @throws If the property doesn't exist or is not an object
  #[wasm_bindgen(js_name = getIfObjectOrThrow)]
  pub fn get_if_object_or_throw(
    &self,
    name: &str,
  ) -> Result<JsonObject, JsValue> {
    self.get_if_object(name)
      .ok_or_else(|| throw_error(&format!("Expected property '{}' to have an object value, but it was not found or has a different type", name)))
  }

  /// Gets a property value as an object, creating an empty object if the property doesn't exist.
  /// Returns undefined if the property exists but has a non-object value.
  /// @param name - The property name to get
  /// @returns The object value, or undefined if property has a non-object value
  #[wasm_bindgen(js_name = getIfObjectOrCreate)]
  pub fn get_if_object_or_create(&self, name: &str) -> Option<JsonObject> {
    self
      .inner
      .object_value_or_create(name)
      .map(|o| JsonObject { inner: o })
  }

  /// Gets a property value as an object, creating or replacing the value with an empty object if needed.
  /// Unlike getIfObjectOrCreate, this always returns an object by replacing non-object values.
  /// @param name - The property name to get
  /// @returns The object value (always succeeds)
  #[wasm_bindgen(js_name = getIfObjectOrForce)]
  pub fn get_if_object_or_force(&self, name: &str) -> JsonObject {
    JsonObject {
      inner: self.inner.object_value_or_set(name),
    }
  }

  /// Gets a property value if it's an array.
  /// @param name - The property name to look up
  /// @returns The array value, or undefined if property doesn't exist or is not an array
  #[wasm_bindgen(js_name = getIfArray)]
  pub fn get_if_array(&self, name: &str) -> Option<JsonArray> {
    self.inner.array_value(name).map(|a| JsonArray { inner: a })
  }

  /// Gets a property value as an array, throwing if not found or wrong type.
  /// @param name - The property name to look up
  /// @returns The array value
  /// @throws If the property doesn't exist or is not an array
  #[wasm_bindgen(js_name = getIfArrayOrThrow)]
  pub fn get_if_array_or_throw(
    &self,
    name: &str,
  ) -> Result<JsonArray, JsValue> {
    self.get_if_array(name)
      .ok_or_else(|| throw_error(&format!("Expected property '{}' to have an array value, but it was not found or has a different type", name)))
  }

  /// Gets a property value as an array, creating an empty array if the property doesn't exist.
  /// Returns undefined if the property exists but has a non-array value.
  /// @param name - The property name to get
  /// @returns The array value, or undefined if property has a non-array value
  #[wasm_bindgen(js_name = getIfArrayOrCreate)]
  pub fn get_if_array_or_create(&self, name: &str) -> Option<JsonArray> {
    self
      .inner
      .array_value_or_create(name)
      .map(|a| JsonArray { inner: a })
  }

  /// Gets a property value as an array, creating or replacing the value with an empty array if needed.
  /// Unlike getIfArrayOrCreate, this always returns an array by replacing non-array values.
  /// @param name - The property name to get
  /// @returns The array value (always succeeds)
  #[wasm_bindgen(js_name = getIfArrayOrForce)]
  pub fn get_if_array_or_force(&self, name: &str) -> JsonArray {
    JsonArray {
      inner: self.inner.array_value_or_set(name),
    }
  }

  /// Removes this object from its parent.
  /// After calling this method, the object is detached from the CST and can no longer be used.
  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  /// Returns all child nodes including whitespace and punctuation.
  /// @returns Array of all child nodes
  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Appends a new property to the object.
  /// @param key - The name of the property to add
  /// @param value - The value to set for the property
  /// @returns The newly created property
  #[wasm_bindgen(js_name = append)]
  pub fn append(
    &self,
    key: &str,
    value: JsValue,
  ) -> Result<ObjectProp, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    let prop = self.inner.append(key, cst_input);
    Ok(ObjectProp { inner: prop })
  }

  /// Inserts a new property at the specified index.
  /// @param index - The position to insert the property at
  /// @param key - The name of the property to add
  /// @param value - The value to set for the property
  /// @returns The newly created property
  #[wasm_bindgen(js_name = insert)]
  pub fn insert(
    &self,
    index: usize,
    key: &str,
    value: JsValue,
  ) -> Result<ObjectProp, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    let prop = self.inner.insert(index, key, cst_input);
    Ok(ObjectProp { inner: prop })
  }

  /// Configures whether trailing commas should be used in this object.
  /// When enabled, trailing commas are added for multiline formatting.
  /// @param enabled - Whether to enable trailing commas
  #[wasm_bindgen(js_name = setTrailingCommas)]
  pub fn set_trailing_commas(&self, enabled: bool) {
    use jsonc_parser::cst::TrailingCommaMode;
    let mode = if enabled {
      TrailingCommaMode::IfMultiline
    } else {
      TrailingCommaMode::Never
    };
    self.inner.set_trailing_commas(mode);
  }

  /// Ensures the object is formatted with each property on its own line.
  #[wasm_bindgen(js_name = ensureMultiline)]
  pub fn ensure_multiline(&self) {
    self.inner.ensure_multiline();
  }

  /// Replaces this object with a new value.
  /// @param replacement - The new value to replace this object with
  /// @returns The new node that replaced this one, or undefined if this was the root value
  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(
    &self,
    replacement: JsValue,
  ) -> Result<Option<Node>, JsValue> {
    let cst_input = js_value_to_cst_input(&replacement)?;
    Ok(
      self
        .inner
        .clone()
        .replace_with(cst_input)
        .map(|n| Node { inner: n }),
    )
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  /// Returns child nodes excluding whitespace, comments, and punctuation.
  /// @returns Array of significant child nodes
  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Returns the child node at the specified index.
  /// @param index - The child index
  /// @returns The child node, or undefined if index is out of bounds
  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }
}

/// Represents the name part of an object property in the CST.
/// Can be either a quoted string or an unquoted word literal (when allowLooseObjectPropertyNames is enabled).
#[wasm_bindgen]
#[derive(Clone)]
pub struct ObjectPropName {
  inner: cst::ObjectPropName,
}

#[wasm_bindgen]
impl ObjectPropName {
  /// Returns the decoded property name (unquoted and unescaped).
  /// @returns The decoded property name
  #[wasm_bindgen(js_name = decodedValue)]
  pub fn decoded_value(&self) -> Result<String, JsValue> {
    self.inner.decoded_value().map_err(|e| {
      throw_error(&format!("Failed to decode property name: {}", e))
    })
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }
}

/// Represents an object property (key-value pair) in the CST.
/// Provides methods for accessing and manipulating both the property name and its value.
#[wasm_bindgen]
#[derive(Clone)]
pub struct ObjectProp {
  inner: cst::CstObjectProp,
}

#[wasm_bindgen]
impl ObjectProp {
  /// Returns the property name.
  /// @returns The property name, or undefined if malformed
  #[wasm_bindgen(js_name = name)]
  pub fn name(&self) -> Option<ObjectPropName> {
    self.inner.name().map(|n| ObjectPropName { inner: n })
  }

  /// Returns the property name, throwing if malformed.
  /// @returns The property name
  /// @throws If the property name is malformed
  #[wasm_bindgen(js_name = nameOrThrow)]
  pub fn name_or_throw(&self) -> Result<ObjectPropName, JsValue> {
    self
      .name()
      .ok_or_else(|| throw_error("Expected a property name, but found none"))
  }

  /// Returns the property value.
  /// @returns The property value, or undefined if malformed
  #[wasm_bindgen(js_name = value)]
  pub fn value(&self) -> Option<Node> {
    self.inner.value().map(|v| Node { inner: v })
  }

  /// Returns the property value, throwing if malformed.
  /// @returns The property value
  /// @throws If the property value is malformed
  #[wasm_bindgen(js_name = valueOrThrow)]
  pub fn value_or_throw(&self) -> Result<Node, JsValue> {
    self
      .value()
      .ok_or_else(|| throw_error("Expected a property value, but found none"))
  }

  /// Returns the property value if it's an object.
  /// @returns The object value, or undefined if not an object
  #[wasm_bindgen(js_name = valueIfObject)]
  pub fn value_if_object(&self) -> Option<JsonObject> {
    self.inner.object_value().map(|o| JsonObject { inner: o })
  }

  /// Returns the property value as an object, throwing if not an object.
  /// @returns The object value
  /// @throws If the property value is not an object
  #[wasm_bindgen(js_name = valueIfObjectOrThrow)]
  pub fn value_if_object_or_throw(&self) -> Result<JsonObject, JsValue> {
    self.value_if_object()
      .ok_or_else(|| throw_error("Expected property to have an object value, but it has a different type"))
  }

  /// Gets the property value as an object, replacing the value with an empty object if needed.
  /// Always returns an object by replacing non-object values.
  /// @returns The object value (always succeeds)
  #[wasm_bindgen(js_name = valueIfObjectOrForce)]
  pub fn value_if_object_or_force(&self) -> JsonObject {
    JsonObject {
      inner: self.inner.object_value_or_set(),
    }
  }

  /// Returns the property value if it's an array.
  /// @returns The array value, or undefined if not an array
  #[wasm_bindgen(js_name = valueIfArray)]
  pub fn value_if_array(&self) -> Option<JsonArray> {
    self.inner.array_value().map(|a| JsonArray { inner: a })
  }

  /// Returns the property value as an array, throwing if not an array.
  /// @returns The array value
  /// @throws If the property value is not an array
  #[wasm_bindgen(js_name = valueIfArrayOrThrow)]
  pub fn value_if_array_or_throw(&self) -> Result<JsonArray, JsValue> {
    self.value_if_array().ok_or_else(|| {
      throw_error(
        "Expected property to have an array value, but it has a different type",
      )
    })
  }

  /// Gets the property value as an array, replacing the value with an empty array if needed.
  /// Always returns an array by replacing non-array values.
  /// @returns The array value (always succeeds)
  #[wasm_bindgen(js_name = valueIfArrayOrForce)]
  pub fn value_if_array_or_force(&self) -> JsonArray {
    JsonArray {
      inner: self.inner.array_value_or_set(),
    }
  }

  /// Removes this property from its parent object.
  /// After calling this method, the property is detached from the CST and can no longer be used.
  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  /// Returns the index of this property within its parent object.
  /// @returns The property index
  #[wasm_bindgen(js_name = propertyIndex)]
  pub fn property_index(&self) -> usize {
    self.inner.property_index()
  }

  /// Sets the value of this property.
  /// @param value - The new value to set
  #[wasm_bindgen(js_name = setValue)]
  pub fn set_value(&self, value: JsValue) -> Result<(), JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    self.inner.set_value(cst_input);
    Ok(())
  }

  /// Replaces this property with a new property.
  /// This allows changing both the property name and its value.
  /// @param key - The new property name
  /// @param replacement - The new value for the property
  /// @returns The new node that replaced this property, or undefined if this was the root value
  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(
    &self,
    key: &str,
    replacement: JsValue,
  ) -> Result<Option<Node>, JsValue> {
    let cst_input = js_value_to_cst_input(&replacement)?;
    Ok(
      self
        .inner
        .clone()
        .replace_with(key, cst_input)
        .map(|n| Node { inner: n }),
    )
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the previous property in the same object.
  /// @returns The previous property, or undefined if this is the first property
  #[wasm_bindgen(js_name = previousProperty)]
  pub fn previous_property(&self) -> Option<ObjectProp> {
    self
      .inner
      .previous_property()
      .map(|p| ObjectProp { inner: p })
  }

  /// Returns the next property in the same object.
  /// @returns The next property, or undefined if this is the last property
  #[wasm_bindgen(js_name = nextProperty)]
  pub fn next_property(&self) -> Option<ObjectProp> {
    self.inner.next_property().map(|p| ObjectProp { inner: p })
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  /// Returns all child nodes including whitespace and punctuation.
  /// @returns Array of all child nodes
  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Returns child nodes excluding whitespace, comments, and punctuation.
  /// @returns Array of significant child nodes
  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Returns the child node at the specified index.
  /// @param index - The child index
  /// @returns The child node, or undefined if index is out of bounds
  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }
}

/// Represents a JSON array node in the CST.
/// Provides methods for manipulating array elements.
#[wasm_bindgen]
#[derive(Clone)]
pub struct JsonArray {
  inner: cst::CstArray,
}

#[wasm_bindgen]
impl JsonArray {
  /// Returns all element nodes in the array.
  /// @returns Array of element nodes
  #[wasm_bindgen(js_name = elements)]
  pub fn elements(&self) -> Vec<Node> {
    self
      .inner
      .elements()
      .into_iter()
      .map(|e| Node { inner: e })
      .collect()
  }

  /// Removes this array from its parent.
  /// After calling this method, the array is detached from the CST and can no longer be used.
  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  /// Ensures the array is formatted with each element on its own line.
  #[wasm_bindgen(js_name = ensureMultiline)]
  pub fn ensure_multiline(&self) {
    self.inner.ensure_multiline();
  }

  /// Returns all child nodes including whitespace and punctuation.
  /// @returns Array of all child nodes
  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Appends a new element to the end of the array.
  /// @param value - The value to append
  /// @returns The newly created element node
  #[wasm_bindgen(js_name = append)]
  pub fn append(&self, value: JsValue) -> Result<Node, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    let node = self.inner.append(cst_input);
    Ok(Node { inner: node })
  }

  /// Inserts a new element at the specified index.
  /// @param index - The position to insert at
  /// @param value - The value to insert
  /// @returns The newly created element node
  #[wasm_bindgen(js_name = insert)]
  pub fn insert(&self, index: usize, value: JsValue) -> Result<Node, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    let node = self.inner.insert(index, cst_input);
    Ok(Node { inner: node })
  }

  /// Configures whether trailing commas should be used in this array.
  /// When enabled, trailing commas are added for multiline formatting.
  /// @param enabled - Whether to enable trailing commas
  #[wasm_bindgen(js_name = setTrailingCommas)]
  pub fn set_trailing_commas(&self, enabled: bool) {
    use jsonc_parser::cst::TrailingCommaMode;
    let mode = if enabled {
      TrailingCommaMode::IfMultiline
    } else {
      TrailingCommaMode::Never
    };
    self.inner.set_trailing_commas(mode);
  }

  /// Replaces this array with a new value.
  /// @param value - The new value to replace this array with
  /// @returns The new node that replaced this one, or undefined if this was the root value
  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(&self, value: JsValue) -> Result<Option<Node>, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    Ok(
      self
        .inner
        .clone()
        .replace_with(cst_input)
        .map(|n| Node { inner: n }),
    )
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  /// Returns child nodes excluding whitespace, comments, and punctuation.
  /// @returns Array of significant child nodes
  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  /// Returns the child node at the specified index.
  /// @param index - The child index
  /// @returns The child node, or undefined if index is out of bounds
  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }
}

/// Represents a string literal node in the CST.
/// Provides methods for manipulating string values and their formatting.
#[wasm_bindgen]
#[derive(Clone)]
pub struct StringLit {
  inner: cst::CstStringLit,
}

#[wasm_bindgen]
impl StringLit {
  /// Returns the decoded string value (without quotes and with escape sequences processed).
  /// @returns The decoded string value
  #[wasm_bindgen(js_name = decodedValue)]
  pub fn decoded_value(&self) -> Result<String, JsValue> {
    self
      .inner
      .decoded_value()
      .map_err(|e| throw_error(&format!("Failed to decode string: {}", e)))
  }

  /// Returns the raw string value including quotes and escape sequences.
  /// @returns The raw string representation
  #[wasm_bindgen(js_name = rawValue)]
  pub fn raw_value(&self) -> String {
    self.inner.raw_value().to_string()
  }

  /// Sets the raw string value (should include quotes).
  /// @param value - The new raw string value
  #[wasm_bindgen(js_name = setRawValue)]
  pub fn set_raw_value(&self, value: String) {
    self.inner.set_raw_value(value);
  }

  /// Replaces this string literal with a new value.
  /// @param replacement - The new value to replace this string with
  /// @returns The new node that replaced this one, or undefined if this was the root value
  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(
    &self,
    replacement: JsValue,
  ) -> Result<Option<Node>, JsValue> {
    let cst_input = js_value_to_cst_input(&replacement)?;
    Ok(
      self
        .inner
        .clone()
        .replace_with(cst_input)
        .map(|n| Node { inner: n }),
    )
  }

  /// Removes this string literal from its parent.
  /// After calling this method, the node is detached from the CST and can no longer be used.
  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }
}

/// Represents a number literal node in the CST.
/// Provides methods for manipulating number values.
#[wasm_bindgen]
#[derive(Clone)]
pub struct NumberLit {
  inner: cst::CstNumberLit,
}

#[wasm_bindgen]
impl NumberLit {
  /// Returns the raw string representation of the number.
  /// Returns a string to preserve the exact formatting (e.g., "1.0" vs "1", "1e10" vs "10000000000").
  /// @returns The number as a string
  #[wasm_bindgen(js_name = value)]
  pub fn value(&self) -> String {
    self.inner.to_string()
  }

  /// Sets the raw number value.
  /// The value should be a valid JSON number string (e.g., "42", "3.14", "1e10").
  /// @param value - The raw number string to set
  #[wasm_bindgen(js_name = setRawValue)]
  pub fn set_raw_value(&self, value: String) {
    self.inner.set_raw_value(value);
  }

  /// Replaces this number literal with a new value.
  /// @param replacement - The new value to replace this number with
  /// @returns The new node that replaced this one, or undefined if this was the root value
  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(
    &self,
    replacement: JsValue,
  ) -> Result<Option<Node>, JsValue> {
    let cst_input = js_value_to_cst_input(&replacement)?;
    Ok(
      self
        .inner
        .clone()
        .replace_with(cst_input)
        .map(|n| Node { inner: n }),
    )
  }

  /// Removes this node from its parent.
  /// After calling this method, the node is detached from the CST and can no longer be used.
  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }
}

/// Represents a boolean literal node in the CST.
/// Provides methods for manipulating boolean values.
#[wasm_bindgen]
#[derive(Clone)]
pub struct BooleanLit {
  inner: cst::CstBooleanLit,
}

#[wasm_bindgen]
impl BooleanLit {
  /// Returns the boolean value (true or false).
  /// @returns The boolean value
  #[wasm_bindgen(js_name = value)]
  pub fn value(&self) -> bool {
    self.inner.value()
  }

  /// Sets the boolean value.
  /// @param value - The new boolean value (true or false)
  #[wasm_bindgen(js_name = setValue)]
  pub fn set_value(&self, value: bool) {
    self.inner.set_value(value);
  }

  /// Replaces this boolean literal with a new value.
  /// @param replacement - The new value to replace this boolean with
  /// @returns The new node that replaced this one, or undefined if this was the root value
  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(
    &self,
    replacement: JsValue,
  ) -> Result<Option<Node>, JsValue> {
    let cst_input = js_value_to_cst_input(&replacement)?;
    Ok(
      self
        .inner
        .clone()
        .replace_with(cst_input)
        .map(|n| Node { inner: n }),
    )
  }

  /// Removes this node from its parent.
  /// After calling this method, the node is detached from the CST and can no longer be used.
  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }
}

/// Represents a null keyword node in the CST.
#[wasm_bindgen]
#[derive(Clone)]
pub struct NullKeyword {
  inner: cst::CstNullKeyword,
}

#[wasm_bindgen]
impl NullKeyword {
  /// Replaces this null keyword with a new value.
  /// @param replacement - The new value to replace this null with
  /// @returns The new node that replaced this one, or undefined if this was the root value
  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(
    &self,
    replacement: JsValue,
  ) -> Result<Option<Node>, JsValue> {
    let cst_input = js_value_to_cst_input(&replacement)?;
    Ok(
      self
        .inner
        .clone()
        .replace_with(cst_input)
        .map(|n| Node { inner: n }),
    )
  }

  /// Removes this node from its parent.
  /// After calling this method, the node is detached from the CST and can no longer be used.
  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }
}

/// Represents an unquoted word literal node in the CST.
/// Used for unquoted property names when `allowLooseObjectPropertyNames` is enabled.
#[wasm_bindgen]
#[derive(Clone)]
pub struct WordLit {
  inner: cst::CstWordLit,
}

#[wasm_bindgen]
impl WordLit {
  /// Returns the unquoted word value.
  /// @returns The word literal as a string
  #[wasm_bindgen(js_name = value)]
  pub fn value(&self) -> String {
    self.inner.to_string()
  }

  /// Sets the raw word value.
  /// The value should be a valid unquoted identifier (alphanumeric and underscores).
  /// @param value - The raw word string to set
  #[wasm_bindgen(js_name = setRawValue)]
  pub fn set_raw_value(&self, value: String) {
    self.inner.set_raw_value(value);
  }

  /// Replaces this word literal with a new value.
  /// @param replacement - The new value to replace this word with
  /// @returns The new node that replaced this one, or undefined if this was the root value
  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(
    &self,
    replacement: JsValue,
  ) -> Result<Option<Node>, JsValue> {
    let cst_input = js_value_to_cst_input(&replacement)?;
    Ok(
      self
        .inner
        .clone()
        .replace_with(cst_input)
        .map(|n| Node { inner: n }),
    )
  }

  /// Removes this node from its parent.
  /// After calling this method, the node is detached from the CST and can no longer be used.
  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  /// Returns the parent node in the CST.
  /// @returns The parent node, or undefined if this is the root
  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  /// Returns all ancestor nodes from parent to root.
  /// @returns Array of ancestor nodes
  #[wasm_bindgen(js_name = ancestors)]
  pub fn ancestors(&self) -> Vec<Node> {
    self
      .inner
      .ancestors()
      .map(|a| Node {
        inner: JsoncCstNode::Container(a),
      })
      .collect()
  }

  /// Returns the index of this node within its parent's children.
  /// @returns The child index
  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  /// Returns the previous sibling node.
  /// @returns The previous sibling, or undefined if this is the first child
  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  /// Returns all previous sibling nodes.
  /// @returns Array of previous siblings
  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the next sibling node.
  /// @returns The next sibling, or undefined if this is the last child
  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  /// Returns all next sibling nodes.
  /// @returns Array of next siblings
  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  /// Returns the root node of the document.
  /// @returns The root node, or undefined if detached
  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  /// Returns the indentation string used at this node's depth.
  /// @returns The indentation string, or undefined if not applicable
  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  /// Returns whether this node's container uses trailing commas.
  /// @returns true if trailing commas are used
  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }
}
