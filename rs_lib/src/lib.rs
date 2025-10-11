use jsonc_parser::cst;
use jsonc_parser::cst::CstContainerNode;
use jsonc_parser::cst::CstLeafNode;
use jsonc_parser::cst::CstNode as JsoncCstNode;
use jsonc_parser::ParseOptions;
use wasm_bindgen::prelude::*;
use jsonc_parser::cst::CstInputValue;

fn throw_error(msg: &str) -> JsValue {
  js_sys::Error::new(msg).into()
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(typescript_type = "{ allowComments?: boolean; allowTrailingCommas?: boolean; allowLooseObjectPropertyNames?: boolean; }")]
  pub type JsoncParseOptionsObject;
}

fn parse_options_from_js(obj: &JsValue) -> ParseOptions {
  let mut options = ParseOptions::default();

  if obj.is_object() {
    if let Ok(allow_comments) = js_sys::Reflect::get(obj, &"allowComments".into()) {
      if let Some(val) = allow_comments.as_bool() {
        options.allow_comments = val;
      }
    }

    if let Ok(allow_trailing_commas) =
      js_sys::Reflect::get(obj, &"allowTrailingCommas".into())
    {
      if let Some(val) = allow_trailing_commas.as_bool() {
        options.allow_trailing_commas = val;
      }
    }

    if let Ok(allow_loose) =
      js_sys::Reflect::get(obj, &"allowLooseObjectPropertyNames".into())
    {
      if let Some(val) = allow_loose.as_bool() {
        options.allow_loose_object_property_names = val;
      }
    }
  }

  options
}

fn js_value_to_cst_input(value: &JsValue) -> Result<CstInputValue, JsValue> {
  // Convert JsValue to serde_json::Value using serde-wasm-bindgen
  let serde_value: serde_json::Value = serde_wasm_bindgen::from_value(value.clone())
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
      let converted: Vec<CstInputValue> = arr.into_iter().map(convert_serde_to_cst_input).collect();
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

#[wasm_bindgen]
pub struct RootNode {
  inner: cst::CstRootNode,
}

#[wasm_bindgen]
impl RootNode {
  #[wasm_bindgen(js_name = parse)]
  pub fn parse(text: &str, options: Option<JsoncParseOptionsObject>) -> Result<RootNode, JsValue> {
    let parse_options = match options {
      Some(opts) => parse_options_from_js(&opts.into()),
      None => ParseOptions::default(),
    };

    let root = cst::CstRootNode::parse(text, &parse_options)
      .map_err(|e| throw_error(&format!("Parse error: {}", e.kind())))?;
    Ok(RootNode { inner: root })
  }

  #[wasm_bindgen(js_name = value)]
  pub fn value(&self) -> Option<Node> {
    self.inner.value().map(|v| Node { inner: v })
  }

  #[wasm_bindgen(js_name = valueOrThrow)]
  pub fn value_or_throw(&self) -> Result<Node, JsValue> {
    self.value()
      .ok_or_else(|| throw_error("Expected a value, but found none"))
  }

  #[wasm_bindgen(js_name = objectValue)]
  pub fn object_value(&self) -> Option<JsonObject> {
    self.inner.object_value().map(|o| JsonObject { inner: o })
  }

  #[wasm_bindgen(js_name = objectValueOrThrow)]
  pub fn object_value_or_throw(&self) -> Result<JsonObject, JsValue> {
    self.object_value()
      .ok_or_else(|| throw_error("Expected an object value, but found a different type"))
  }

  #[wasm_bindgen(js_name = objectValueOrCreate)]
  pub fn object_value_or_create(&self) -> Option<JsonObject> {
    self.inner.object_value_or_create().map(|o| JsonObject { inner: o })
  }

  #[wasm_bindgen(js_name = objectValueOrSet)]
  pub fn object_value_or_set(&self) -> JsonObject {
    JsonObject {
      inner: self.inner.object_value_or_set(),
    }
  }

  #[wasm_bindgen(js_name = arrayValue)]
  pub fn array_value(&self) -> Option<JsonArray> {
    self.inner.array_value().map(|a| JsonArray { inner: a })
  }

  #[wasm_bindgen(js_name = arrayValueOrThrow)]
  pub fn array_value_or_throw(&self) -> Result<JsonArray, JsValue> {
    self.array_value()
      .ok_or_else(|| throw_error("Expected an array value, but found a different type"))
  }

  #[wasm_bindgen(js_name = arrayValueOrCreate)]
  pub fn array_value_or_create(&self) -> Option<JsonArray> {
    self.inner.array_value_or_create().map(|a| JsonArray { inner: a })
  }

  #[wasm_bindgen(js_name = arrayValueOrSet)]
  pub fn array_value_or_set(&self) -> JsonArray {
    JsonArray {
      inner: self.inner.array_value_or_set(),
    }
  }

  #[wasm_bindgen(js_name = toString)]
  pub fn to_string_output(&self) -> String {
    self.inner.to_string()
  }

  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = setValue)]
  pub fn set_value(&self, root_value: JsValue) -> Result<(), JsValue> {
    let cst_input = js_value_to_cst_input(&root_value)?;
    self.inner.set_value(cst_input);
    Ok(())
  }

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

  #[wasm_bindgen(js_name = clearChildren)]
  pub fn clear_children(&self) {
    self.inner.clear_children();
  }

  #[wasm_bindgen(js_name = singleIndentText)]
  pub fn single_indent_text(&self) -> Option<String> {
    self.inner.single_indent_text()
  }

  #[wasm_bindgen(js_name = newlineKind)]
  pub fn newline_kind(&self) -> String {
    format!("{:?}", self.inner.newline_kind())
  }

  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

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

  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Node {
  inner: JsoncCstNode,
}

#[wasm_bindgen]
impl Node {
  #[wasm_bindgen(js_name = isContainer)]
  pub fn is_container(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Container(_))
  }

  #[wasm_bindgen(js_name = isLeaf)]
  pub fn is_leaf(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(_))
  }

  #[wasm_bindgen(js_name = asObject)]
  pub fn as_object(&self) -> Option<JsonObject> {
    match &self.inner {
      JsoncCstNode::Container(CstContainerNode::Object(obj)) => {
        Some(JsonObject { inner: obj.clone() })
      }
      _ => None,
    }
  }

  #[wasm_bindgen(js_name = asObjectOrThrow)]
  pub fn as_object_or_throw(&self) -> Result<JsonObject, JsValue> {
    match &self.inner {
      JsoncCstNode::Container(CstContainerNode::Object(obj)) => {
        Ok(JsonObject { inner: obj.clone() })
      }
      _ => Err(throw_error("Expected an object node, but found a different type")),
    }
  }

  #[wasm_bindgen(js_name = asArray)]
  pub fn as_array(&self) -> Option<JsonArray> {
    match &self.inner {
      JsoncCstNode::Container(CstContainerNode::Array(arr)) => {
        Some(JsonArray { inner: arr.clone() })
      }
      _ => None,
    }
  }

  #[wasm_bindgen(js_name = asArrayOrThrow)]
  pub fn as_array_or_throw(&self) -> Result<JsonArray, JsValue> {
    match &self.inner {
      JsoncCstNode::Container(CstContainerNode::Array(arr)) => {
        Ok(JsonArray { inner: arr.clone() })
      }
      _ => Err(throw_error("Expected an array node, but found a different type")),
    }
  }

  #[wasm_bindgen(js_name = asRootNode)]
  pub fn as_root_node(&self) -> Option<RootNode> {
    self.inner.as_root_node().map(|r| RootNode { inner: r })
  }

  #[wasm_bindgen(js_name = asRootNodeOrThrow)]
  pub fn as_root_node_or_throw(&self) -> Result<RootNode, JsValue> {
    self.as_root_node()
      .ok_or_else(|| throw_error("Expected a root node, but found a different type"))
  }

  #[wasm_bindgen(js_name = asString)]
  pub fn as_string(&self) -> Option<String> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::StringLit(s)) => s.decoded_value().ok(),
      _ => None,
    }
  }

  #[wasm_bindgen(js_name = asStringOrThrow)]
  pub fn as_string_or_throw(&self) -> Result<String, JsValue> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::StringLit(s)) => s
        .decoded_value()
        .map_err(|e| throw_error(&format!("Failed to decode string: {}", e))),
      _ => Err(throw_error("Expected a string node, but found a different type")),
    }
  }

  #[wasm_bindgen(js_name = numberValue)]
  pub fn number_value(&self) -> Option<String> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::NumberLit(n)) => Some(n.to_string()),
      _ => None,
    }
  }

  #[wasm_bindgen(js_name = numberValueOrThrow)]
  pub fn number_value_or_throw(&self) -> Result<String, JsValue> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::NumberLit(n)) => Ok(n.to_string()),
      _ => Err(throw_error("Expected a number node, but found a different type")),
    }
  }

  #[wasm_bindgen(js_name = asBoolean)]
  pub fn as_boolean(&self) -> Option<bool> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::BooleanLit(b)) => Some(b.value()),
      _ => None,
    }
  }

  #[wasm_bindgen(js_name = asBooleanOrThrow)]
  pub fn as_boolean_or_throw(&self) -> Result<bool, JsValue> {
    match &self.inner {
      JsoncCstNode::Leaf(CstLeafNode::BooleanLit(b)) => Ok(b.value()),
      _ => Err(throw_error("Expected a boolean node, but found a different type")),
    }
  }

  #[wasm_bindgen(js_name = isNull)]
  pub fn is_null(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(CstLeafNode::NullKeyword(_)))
  }

  #[wasm_bindgen(js_name = isString)]
  pub fn is_string(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(CstLeafNode::StringLit(_)))
  }

  #[wasm_bindgen(js_name = isNumber)]
  pub fn is_number(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(CstLeafNode::NumberLit(_)))
  }

  #[wasm_bindgen(js_name = isBoolean)]
  pub fn is_boolean(&self) -> bool {
    matches!(self.inner, JsoncCstNode::Leaf(CstLeafNode::BooleanLit(_)))
  }

  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  #[wasm_bindgen(js_name = parentOrThrow)]
  pub fn parent_or_throw(&self) -> Result<Node, JsValue> {
    self.parent()
      .ok_or_else(|| throw_error("Expected a parent node, but found none"))
  }

  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

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

  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  #[wasm_bindgen(js_name = rootNodeOrThrow)]
  pub fn root_node_or_throw(&self) -> Result<RootNode, JsValue> {
    self.root_node()
      .ok_or_else(|| throw_error("Expected a root node, but found none"))
  }

  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  #[wasm_bindgen(js_name = isTrivia)]
  pub fn is_trivia(&self) -> bool {
    self.inner.is_trivia()
  }

  #[wasm_bindgen(js_name = isNewline)]
  pub fn is_newline(&self) -> bool {
    self.inner.is_newline()
  }

  #[wasm_bindgen(js_name = isComma)]
  pub fn is_comma(&self) -> bool {
    self.inner.is_comma()
  }

  #[wasm_bindgen(js_name = isComment)]
  pub fn is_comment(&self) -> bool {
    self.inner.is_comment()
  }

  #[wasm_bindgen(js_name = isToken)]
  pub fn is_token(&self) -> bool {
    self.inner.is_token()
  }

  #[wasm_bindgen(js_name = isWhitespace)]
  pub fn is_whitespace(&self) -> bool {
    self.inner.is_whitespace()
  }

  #[wasm_bindgen(js_name = tokenChar)]
  pub fn token_char(&self) -> Option<String> {
    self.inner.token_char().map(|c| c.to_string())
  }

  #[wasm_bindgen(js_name = elementIndex)]
  pub fn element_index(&self) -> Option<usize> {
    self.inner.element_index()
  }

  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JsonObject {
  inner: cst::CstObject,
}

#[wasm_bindgen]
impl JsonObject {
  #[wasm_bindgen(js_name = properties)]
  pub fn properties(&self) -> Vec<ObjectProp> {
    self
      .inner
      .properties()
      .into_iter()
      .map(|p| ObjectProp { inner: p })
      .collect()
  }

  #[wasm_bindgen(js_name = get)]
  pub fn get(&self, key: &str) -> Option<ObjectProp> {
    self.inner.get(key).map(|p| ObjectProp { inner: p })
  }

  #[wasm_bindgen(js_name = getOrThrow)]
  pub fn get_or_throw(&self, key: &str) -> Result<ObjectProp, JsValue> {
    self.get(key)
      .ok_or_else(|| throw_error(&format!("Expected property '{}', but it was not found", key)))
  }

  #[wasm_bindgen(js_name = objectValue)]
  pub fn object_value(&self, name: &str) -> Option<JsonObject> {
    self
      .inner
      .object_value(name)
      .map(|o| JsonObject { inner: o })
  }

  #[wasm_bindgen(js_name = objectValueOrThrow)]
  pub fn object_value_or_throw(&self, name: &str) -> Result<JsonObject, JsValue> {
    self.object_value(name)
      .ok_or_else(|| throw_error(&format!("Expected property '{}' to have an object value, but it was not found or has a different type", name)))
  }

  #[wasm_bindgen(js_name = objectValueOrCreate)]
  pub fn object_value_or_create(&self, name: &str) -> Option<JsonObject> {
    self
      .inner
      .object_value_or_create(name)
      .map(|o| JsonObject { inner: o })
  }

  #[wasm_bindgen(js_name = objectValueOrSet)]
  pub fn object_value_or_set(&self, name: &str) -> JsonObject {
    JsonObject {
      inner: self.inner.object_value_or_set(name),
    }
  }

  #[wasm_bindgen(js_name = arrayValue)]
  pub fn array_value(&self, name: &str) -> Option<JsonArray> {
    self.inner.array_value(name).map(|a| JsonArray { inner: a })
  }

  #[wasm_bindgen(js_name = arrayValueOrThrow)]
  pub fn array_value_or_throw(&self, name: &str) -> Result<JsonArray, JsValue> {
    self.array_value(name)
      .ok_or_else(|| throw_error(&format!("Expected property '{}' to have an array value, but it was not found or has a different type", name)))
  }

  #[wasm_bindgen(js_name = arrayValueOrCreate)]
  pub fn array_value_or_create(&self, name: &str) -> Option<JsonArray> {
    self
      .inner
      .array_value_or_create(name)
      .map(|a| JsonArray { inner: a })
  }

  #[wasm_bindgen(js_name = arrayValueOrSet)]
  pub fn array_value_or_set(&self, name: &str) -> JsonArray {
    JsonArray {
      inner: self.inner.array_value_or_set(name),
    }
  }

  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = append)]
  pub fn append(&self, prop_name: &str, value: JsValue) -> Result<ObjectProp, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    let prop = self.inner.append(prop_name, cst_input);
    Ok(ObjectProp { inner: prop })
  }

  #[wasm_bindgen(js_name = insert)]
  pub fn insert(&self, index: usize, prop_name: &str, value: JsValue) -> Result<ObjectProp, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    let prop = self.inner.insert(index, prop_name, cst_input);
    Ok(ObjectProp { inner: prop })
  }

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

  #[wasm_bindgen(js_name = ensureMultiline)]
  pub fn ensure_multiline(&self) {
    self.inner.ensure_multiline();
  }

  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(&self, replacement: &str) -> Option<Node> {
    self
      .inner
      .clone()
      .replace_with(replacement.into())
      .map(|n| Node { inner: n })
  }

  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

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

  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct ObjectProp {
  inner: cst::CstObjectProp,
}

#[wasm_bindgen]
impl ObjectProp {
  #[wasm_bindgen(js_name = name)]
  pub fn name(&self) -> Option<String> {
    self.inner.name().and_then(|n| n.decoded_value().ok())
  }

  #[wasm_bindgen(js_name = nameOrThrow)]
  pub fn name_or_throw(&self) -> Result<String, JsValue> {
    self.name()
      .ok_or_else(|| throw_error("Expected a property name, but found none"))
  }

  #[wasm_bindgen(js_name = value)]
  pub fn value(&self) -> Option<Node> {
    self.inner.value().map(|v| Node { inner: v })
  }

  #[wasm_bindgen(js_name = valueOrThrow)]
  pub fn value_or_throw(&self) -> Result<Node, JsValue> {
    self.value()
      .ok_or_else(|| throw_error("Expected a property value, but found none"))
  }

  #[wasm_bindgen(js_name = objectValue)]
  pub fn object_value(&self) -> Option<JsonObject> {
    self.inner.object_value().map(|o| JsonObject { inner: o })
  }

  #[wasm_bindgen(js_name = objectValueOrThrow)]
  pub fn object_value_or_throw(&self) -> Result<JsonObject, JsValue> {
    self.object_value()
      .ok_or_else(|| throw_error("Expected property to have an object value, but it has a different type"))
  }

  #[wasm_bindgen(js_name = objectValueOrSet)]
  pub fn object_value_or_set(&self) -> JsonObject {
    JsonObject {
      inner: self.inner.object_value_or_set(),
    }
  }

  #[wasm_bindgen(js_name = arrayValue)]
  pub fn array_value(&self) -> Option<JsonArray> {
    self.inner.array_value().map(|a| JsonArray { inner: a })
  }

  #[wasm_bindgen(js_name = arrayValueOrThrow)]
  pub fn array_value_or_throw(&self) -> Result<JsonArray, JsValue> {
    self.array_value()
      .ok_or_else(|| throw_error("Expected property to have an array value, but it has a different type"))
  }

  #[wasm_bindgen(js_name = arrayValueOrSet)]
  pub fn array_value_or_set(&self) -> JsonArray {
    JsonArray {
      inner: self.inner.array_value_or_set(),
    }
  }

  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  #[wasm_bindgen(js_name = propertyIndex)]
  pub fn property_index(&self) -> usize {
    self.inner.property_index()
  }

  #[wasm_bindgen(js_name = setValue)]
  pub fn set_value(&self, value: JsValue) -> Result<(), JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    self.inner.set_value(cst_input);
    Ok(())
  }

  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(&self, key: &str, replacement: &str) -> Option<Node> {
    self
      .inner
      .clone()
      .replace_with(key, replacement.into())
      .map(|n| Node { inner: n })
  }

  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

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

  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = previousProperty)]
  pub fn previous_property(&self) -> Option<ObjectProp> {
    self
      .inner
      .previous_property()
      .map(|p| ObjectProp { inner: p })
  }

  #[wasm_bindgen(js_name = nextProperty)]
  pub fn next_property(&self) -> Option<ObjectProp> {
    self.inner.next_property().map(|p| ObjectProp { inner: p })
  }

  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JsonArray {
  inner: cst::CstArray,
}

#[wasm_bindgen]
impl JsonArray {
  #[wasm_bindgen(js_name = elements)]
  pub fn elements(&self) -> Vec<Node> {
    self
      .inner
      .elements()
      .into_iter()
      .map(|e| Node { inner: e })
      .collect()
  }

  #[wasm_bindgen(js_name = remove)]
  pub fn remove(self) {
    self.inner.remove();
  }

  #[wasm_bindgen(js_name = ensureMultiline)]
  pub fn ensure_multiline(&self) {
    self.inner.ensure_multiline();
  }

  #[wasm_bindgen(js_name = children)]
  pub fn children(&self) -> Vec<Node> {
    self
      .inner
      .children()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = append)]
  pub fn append(&self, value: JsValue) -> Result<Node, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    let node = self.inner.append(cst_input);
    Ok(Node { inner: node })
  }

  #[wasm_bindgen(js_name = insert)]
  pub fn insert(&self, index: usize, value: JsValue) -> Result<Node, JsValue> {
    let cst_input = js_value_to_cst_input(&value)?;
    let node = self.inner.insert(index, cst_input);
    Ok(Node { inner: node })
  }

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

  #[wasm_bindgen(js_name = replaceWith)]
  pub fn replace_with(&self, replacement: &str) -> Option<Node> {
    self
      .inner
      .clone()
      .replace_with(replacement.into())
      .map(|n| Node { inner: n })
  }

  #[wasm_bindgen(js_name = parent)]
  pub fn parent(&self) -> Option<Node> {
    self.inner.parent().map(|p| Node {
      inner: JsoncCstNode::Container(p),
    })
  }

  #[wasm_bindgen(js_name = childIndex)]
  pub fn child_index(&self) -> usize {
    self.inner.child_index()
  }

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

  #[wasm_bindgen(js_name = previousSibling)]
  pub fn previous_sibling(&self) -> Option<Node> {
    self.inner.previous_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = previousSiblings)]
  pub fn previous_siblings(&self) -> Vec<Node> {
    self
      .inner
      .previous_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = nextSibling)]
  pub fn next_sibling(&self) -> Option<Node> {
    self.inner.next_sibling().map(|s| Node { inner: s })
  }

  #[wasm_bindgen(js_name = nextSiblings)]
  pub fn next_siblings(&self) -> Vec<Node> {
    self
      .inner
      .next_siblings()
      .map(|s| Node { inner: s })
      .collect()
  }

  #[wasm_bindgen(js_name = rootNode)]
  pub fn root_node(&self) -> Option<RootNode> {
    self.inner.root_node().map(|r| RootNode { inner: r })
  }

  #[wasm_bindgen(js_name = indentText)]
  pub fn indent_text(&self) -> Option<String> {
    self.inner.indent_text().map(|s| s.to_string())
  }

  #[wasm_bindgen(js_name = usesTrailingCommas)]
  pub fn uses_trailing_commas(&self) -> bool {
    self.inner.uses_trailing_commas()
  }

  #[wasm_bindgen(js_name = childrenExcludeTriviaAndTokens)]
  pub fn children_exclude_trivia_and_tokens(&self) -> Vec<Node> {
    self
      .inner
      .children_exclude_trivia_and_tokens()
      .into_iter()
      .map(|n| Node { inner: n })
      .collect()
  }

  #[wasm_bindgen(js_name = childAtIndex)]
  pub fn child_at_index(&self, index: usize) -> Option<Node> {
    self.inner.child_at_index(index).map(|n| Node { inner: n })
  }
}
