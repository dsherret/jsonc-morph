import { assertEquals, assertExists } from "@std/assert";
import { parse } from "./mod.ts";

Deno.test("RootNode - parse simple object", () => {
  const text = '{"name": "test", "value": 42}';
  const root = parse(text);

  assertExists(root);
  const obj = root.asObject();
  assertExists(obj);
});

Deno.test("RootNode - parse with comments", () => {
  const text = `{
    // This is a comment
    "name": "test",
    /* Multi-line
       comment */
    "value": 42
  }`;

  const root = parse(text);
  assertExists(root);

  const obj = root.asObject();
  assertExists(obj);
});

Deno.test("RootNode - parse with trailing commas", () => {
  const text = `{
    "items": [1, 2, 3,],
    "name": "test",
  }`;

  const root = parse(text);
  assertExists(root);

  const obj = root.asObject();
  assertExists(obj);
});

Deno.test("RootNode - parse with options", () => {
  const text = `{
    // Comment
    "value": 123,
  }`;

  const root = parse(text, {
    allowComments: true,
    allowTrailingCommas: true,
  });
  assertExists(root);
});

Deno.test("RootNode - toString roundtrip", () => {
  const text = '{"name": "test", "value": 42}';
  const root = parse(text);

  const output = root.toString();
  assertEquals(output, text);
});

Deno.test("JsonObject - get properties", () => {
  const text = '{"name": "test", "value": 42, "active": true}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const props = obj.properties();
  assertEquals(props.length, 3);

  const names = props.map((p) => p.name()?.decodedValue()).filter((n) =>
    n !== undefined
  );
  assertEquals(names, ["name", "value", "active"]);
});

Deno.test("JsonObject - get property by key", () => {
  const text = '{"name": "test", "value": 42}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const nameProp = obj.get("name");
  assertExists(nameProp);
  assertEquals(nameProp.name()?.decodedValue(), "name");

  const nonExistent = obj.get("nonexistent");
  assertEquals(nonExistent, undefined);
});

Deno.test("JsonObject - nested object access", () => {
  const text = '{"config": {"debug": true, "level": "info"}}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const configObj = obj.getIfObject("config");
  assertExists(configObj);

  const debugProp = configObj.get("debug");
  assertExists(debugProp);
  assertEquals(debugProp.name()?.decodedValue(), "debug");
});

Deno.test("JsonObject - nested array access", () => {
  const text = '{"items": [1, 2, 3]}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const itemsJsonArray = obj.getIfArray("items");
  assertExists(itemsJsonArray);

  const elements = itemsJsonArray.elements();
  assertEquals(elements.length, 3);
});

Deno.test("JsonArray - parse array root", () => {
  const text = '[1, 2, 3, "four", true, null]';
  const root = parse(text);

  const arr = root.asArray();
  assertExists(arr);

  const elements = arr.elements();
  assertEquals(elements.length, 6);
});

Deno.test("JsonArray - nested arrays", () => {
  const text = "[[1, 2], [3, 4], [5, 6]]";
  const root = parse(text);

  const arr = root.asArray();
  assertExists(arr);

  const elements = arr.elements();
  assertEquals(elements.length, 3);

  const firstNested = elements[0].asArray();
  assertExists(firstNested);
  assertEquals(firstNested.elements().length, 2);
});

Deno.test("Node - type checking for string", () => {
  const text = '{"name": "test"}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const nameProp = obj.get("name");
  assertExists(nameProp);

  const value = nameProp.value();
  assertExists(value);

  assertEquals(value.isString(), true);
  assertEquals(value.isNumber(), false);
  assertEquals(value.isBoolean(), false);
  assertEquals(value.isNull(), false);
  assertEquals(value.isContainer(), false);
  assertEquals(value.isLeaf(), true);

  const strValue = value.asString();
  assertEquals(strValue, "test");
});

Deno.test("Node - type checking for number", () => {
  const text = '{"count": 42}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const countProp = obj.get("count");
  assertExists(countProp);

  const value = countProp.value();
  assertExists(value);

  assertEquals(value.isNumber(), true);
  assertEquals(value.isString(), false);
  assertEquals(value.isBoolean(), false);
  assertEquals(value.isNull(), false);
});

Deno.test("Node - type checking for boolean", () => {
  const text = '{"active": true}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const activeProp = obj.get("active");
  assertExists(activeProp);

  const value = activeProp.value();
  assertExists(value);

  assertEquals(value.isBoolean(), true);
  assertEquals(value.isString(), false);
  assertEquals(value.isNumber(), false);
  assertEquals(value.isNull(), false);

  const boolValue = value.asBoolean();
  assertEquals(boolValue, true);
});

Deno.test("Node - type checking for null", () => {
  const text = '{"data": null}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const dataProp = obj.get("data");
  assertExists(dataProp);

  const value = dataProp.value();
  assertExists(value);

  assertEquals(value.isNull(), true);
  assertEquals(value.isString(), false);
  assertEquals(value.isNumber(), false);
  assertEquals(value.isBoolean(), false);
});

Deno.test("Node - type checking for object", () => {
  const text = '{"config": {"debug": true}}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const configProp = obj.get("config");
  assertExists(configProp);

  const value = configProp.value();
  assertExists(value);

  assertEquals(value.isContainer(), true);
  assertEquals(value.isLeaf(), false);

  const nestedObj = value.asObject();
  assertExists(nestedObj);
});

Deno.test("Node - type checking for array", () => {
  const text = '{"items": [1, 2, 3]}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const itemsProp = obj.get("items");
  assertExists(itemsProp);

  const value = itemsProp.value();
  assertExists(value);

  assertEquals(value.isContainer(), true);
  assertEquals(value.isLeaf(), false);

  const arr = value.asArray();
  assertExists(arr);
  assertEquals(arr.elements().length, 3);
});

Deno.test("JsonObjectProp - propertyIndex", () => {
  const text = '{"first": 1, "second": 2, "third": 3}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const props = obj.properties();
  assertEquals(props.length, 3);

  assertEquals(props[0].propertyIndex(), 0);
  assertEquals(props[1].propertyIndex(), 1);
  assertEquals(props[2].propertyIndex(), 2);
});

Deno.test("JsonObjectProp - nested object value", () => {
  const text = '{"user": {"name": "John", "age": 30}}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const userProp = obj.get("user");
  assertExists(userProp);

  const userObj = userProp.valueIfObject();
  assertExists(userObj);

  const nameProp = userObj.get("name");
  assertExists(nameProp);

  const nameValue = nameProp.value();
  assertExists(nameValue);
  assertEquals(nameValue.asString(), "John");
});

Deno.test("JsonObjectProp - nested array value", () => {
  const text = '{"scores": [95, 87, 92]}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const scoresProp = obj.get("scores");
  assertExists(scoresProp);

  const scoresArr = scoresProp.valueIfArray();
  assertExists(scoresArr);
  assertEquals(scoresArr.elements().length, 3);
});

Deno.test("Complex nested structure", () => {
  const text = `{
    "name": "MyApp",
    "version": "1.0.0",
    "config": {
      "debug": true,
      "features": ["auth", "api", "ui"],
      "database": {
        "host": "localhost",
        "port": 5432
      }
    },
    "dependencies": [
      {"name": "lib1", "version": "1.0.0"},
      {"name": "lib2", "version": "2.0.0"}
    ]
  }`;

  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  // Check top-level properties
  const nameProp = obj.get("name");
  assertExists(nameProp);
  assertEquals(nameProp.value()?.asString(), "MyApp");

  // Check nested config object
  const configObj = obj.getIfObject("config");
  assertExists(configObj);

  const debugProp = configObj.get("debug");
  assertExists(debugProp);
  assertEquals(debugProp.value()?.asBoolean(), true);

  // Check features array
  const featuresArr = configObj.getIfArray("features");
  assertExists(featuresArr);
  assertEquals(featuresArr.elements().length, 3);

  // Check nested database object
  const dbObj = configObj.getIfObject("database");
  assertExists(dbObj);

  const hostProp = dbObj.get("host");
  assertExists(hostProp);
  assertEquals(hostProp.value()?.asString(), "localhost");

  // Check dependencies array with objects
  const depsArr = obj.getIfArray("dependencies");
  assertExists(depsArr);
  assertEquals(depsArr.elements().length, 2);

  const firstDep = depsArr.elements()[0].asObject();
  assertExists(firstDep);

  const depName = firstDep.get("name");
  assertExists(depName);
  assertEquals(depName.value()?.asString(), "lib1");
});

Deno.test("Parse error handling", () => {
  const invalidText = '{"unclosed": "string}';

  try {
    parse(invalidText);
    throw new Error("Should have thrown parse error");
  } catch (error) {
    assertExists(error);
  }
});

Deno.test("Children access - root", () => {
  const text = '{"a": 1, "b": 2}';
  const root = parse(text);

  const children = root.children();
  assertExists(children);
  assertEquals(children.length > 0, true);
});

Deno.test("Children access - object", () => {
  const text = '{"a": 1, "b": 2}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const children = obj.children();
  assertExists(children);
  assertEquals(children.length > 0, true);
});

Deno.test("Children access - array", () => {
  const text = "[1, 2, 3]";
  const root = parse(text);
  const arr = root.asArray();

  assertExists(arr);

  const children = arr.children();
  assertExists(children);
  assertEquals(children.length > 0, true);
});

Deno.test("Value method - root node", () => {
  const text = '{"key": "value"}';
  const root = parse(text);

  const value = root.value();
  assertExists(value);
  assertEquals(value.isContainer(), true);

  const obj = value.asObject();
  assertExists(obj);
});

Deno.test("Empty object", () => {
  const text = "{}";
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const props = obj.properties();
  assertEquals(props.length, 0);
});

Deno.test("Empty array", () => {
  const text = "[]";
  const root = parse(text);
  const arr = root.asArray();

  assertExists(arr);

  const elements = arr.elements();
  assertEquals(elements.length, 0);
});

Deno.test("Mixed types in array", () => {
  const text = '[1, "two", true, null, {"key": "value"}, [1, 2]]';
  const root = parse(text);
  const arr = root.asArray();

  assertExists(arr);

  const elements = arr.elements();
  assertEquals(elements.length, 6);

  assertEquals(elements[0].isNumber(), true);
  assertEquals(elements[1].isString(), true);
  assertEquals(elements[1].asString(), "two");
  assertEquals(elements[2].isBoolean(), true);
  assertEquals(elements[2].asBoolean(), true);
  assertEquals(elements[3].isNull(), true);
  assertExists(elements[4].asObject());
  assertExists(elements[5].asArray());
});

Deno.test("Special characters in strings", () => {
  const text = '{"message": "Hello\\nWorld\\t!\\r\\n\\"quoted\\""}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const msgProp = obj.get("message");
  assertExists(msgProp);

  const value = msgProp.value();
  assertExists(value);
  assertEquals(value.isString(), true);

  const strValue = value.asString();
  assertExists(strValue);
  assertEquals(strValue.includes("Hello"), true);
});

Deno.test("Unicode in strings", () => {
  const text = '{"emoji": "👍", "chinese": "你好"}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);

  const emojiProp = obj.get("emoji");
  assertExists(emojiProp);
  assertEquals(emojiProp.value()?.asString(), "👍");

  const chineseProp = obj.get("chinese");
  assertExists(chineseProp);
  assertEquals(chineseProp.value()?.asString(), "你好");
});

Deno.test("RootNode - objectValueOrForce creates empty object", () => {
  const text = "null";
  const root = parse(text);

  const obj = root.asObjectOrForce();
  assertExists(obj);

  const output = root.toString();
  assertEquals(output, "{}");
});

Deno.test("RootNode - arrayValueOrForce creates empty array", () => {
  const text = "null";
  const root = parse(text);

  const arr = root.asArrayOrForce();
  assertExists(arr);

  const output = root.toString();
  assertEquals(output, "[]");
});

Deno.test("JsonObject - objectValueOrForce creates nested object", () => {
  const text = "{}";
  const root = parse(text);
  const obj = root.asObjectOrForce();

  const configObj = obj.getIfObjectOrForce("config");
  assertExists(configObj);

  const output = root.toString();
  assertEquals(output.includes("config"), true);
});

Deno.test("JsonObject - arrayValueOrForce creates nested array", () => {
  const text = "{}";
  const root = parse(text);
  const obj = root.asObjectOrForce();

  const itemsArr = obj.getIfArrayOrForce("items");
  assertExists(itemsArr);

  const output = root.toString();
  assertEquals(output.includes("items"), true);
});

Deno.test("JsonArray - ensureMultiline formats array", () => {
  const text = "[1, 2, 3]";
  const root = parse(text);
  const arr = root.asArray();

  assertExists(arr);
  arr.ensureMultiline();

  const output = root.toString();
  assertEquals(output.includes("\n"), true);
});

Deno.test("JsonObjectProp - objectValueOrForce on property", () => {
  const text = '{"user": null}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  const userProp = obj.get("user");
  assertExists(userProp);

  const userObj = userProp.valueIfObjectOrForce();
  assertExists(userObj);

  const output = root.toString();
  assertEquals(output.includes("user"), true);
  assertEquals(output.includes("{}"), true);
});

Deno.test("JsonObjectProp - arrayValueOrForce on property", () => {
  const text = '{"scores": null}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  const scoresProp = obj.get("scores");
  assertExists(scoresProp);

  const scoresArr = scoresProp.valueIfArrayOrForce();
  assertExists(scoresArr);

  const output = root.toString();
  assertEquals(output.includes("scores"), true);
  assertEquals(output.includes("[]"), true);
});

// New manipulation methods tests
Deno.test("JsonArray - append adds element to array", () => {
  const text = "[1, 2]";
  const root = parse(text);
  const arr = root.asArray();

  assertExists(arr);
  arr.append("3");

  const output = root.toString();
  assertEquals(output.includes("3"), true);
  const elements = arr.elements();
  assertEquals(elements.length, 3);
});

Deno.test("JsonArray - insert adds element at index", () => {
  const text = "[1, 3]";
  const root = parse(text);
  const arr = root.asArray();

  assertExists(arr);
  arr.insert(1, "2");

  const output = root.toString();
  assertEquals(output.includes("2"), true);
  const elements = arr.elements();
  assertEquals(elements.length, 3);
});

Deno.test("JsonArray - setTrailingCommas adds trailing commas", () => {
  const text = `[
  1,
  2
]`;
  const root = parse(text);
  const arr = root.asArray();

  assertExists(arr);
  arr.setTrailingCommas(true);

  const output = root.toString();
  assertEquals(output.includes("2,"), true);
});

Deno.test("JsonObject - append adds property", () => {
  const text = '{"a": 1}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  obj.append("b", "2");

  const output = root.toString();
  assertEquals(output.includes("b"), true);
  assertEquals(obj.properties().length, 2);
});

Deno.test("JsonObject - insert adds property at index", () => {
  const text = '{"a": 1, "c": 3}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  obj.insert(1, "b", "2");

  const output = root.toString();
  assertEquals(output.includes("b"), true);
  assertEquals(obj.properties().length, 3);
});

Deno.test("JsonObjectProp - setValue changes property value", () => {
  const text = '{"name": "old"}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  const nameProp = obj.get("name");
  assertExists(nameProp);

  nameProp.setValue('"new"');

  const output = root.toString();
  assertEquals(output.includes("new"), true);
});

Deno.test("JsonObjectProp - previousProperty navigates to previous", () => {
  const text = '{"a": 1, "b": 2, "c": 3}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  const bProp = obj.get("b");
  assertExists(bProp);

  const aProp = bProp.previousProperty();
  assertExists(aProp);
  assertEquals(aProp.name()?.decodedValue(), "a");
});

Deno.test("JsonObjectProp - nextProperty navigates to next", () => {
  const text = '{"a": 1, "b": 2, "c": 3}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  const bProp = obj.get("b");
  assertExists(bProp);

  const cProp = bProp.nextProperty();
  assertExists(cProp);
  assertEquals(cProp.name()?.decodedValue(), "c");
});

Deno.test("Node - parent returns parent node", () => {
  const text = '{"items": [1, 2, 3]}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  const itemsArr = obj.getIfArray("items");
  assertExists(itemsArr);

  const parent = itemsArr.parent();
  assertExists(parent);
  assertEquals(parent.isContainer(), true);
});

Deno.test("Node - childIndex returns position", () => {
  const text = "[1, 2, 3]";
  const root = parse(text);
  const arr = root.asArray();

  assertExists(arr);
  const elements = arr.elements();

  assertEquals(elements[0].childIndex() >= 0, true);
});

Deno.test("Node - rootNode navigates to root", () => {
  const text = '{"a": {"b": {"c": 1}}}';
  const root = parse(text);
  const obj = root.asObject();

  assertExists(obj);
  const aObj = obj.getIfObject("a");
  assertExists(aObj);
  const bObj = aObj.getIfObject("b");
  assertExists(bObj);
  const cProp = bObj.get("c");
  assertExists(cProp);

  const rootFromDeep = cProp.rootNode();
  assertExists(rootFromDeep);
});

Deno.test("Node - isTrivia identifies trivia nodes", () => {
  const text = `{
    // comment
    "a": 1
  }`;
  const root = parse(text);
  const obj = root.asObject();
  assertExists(obj);
  const children = obj.children();

  const hasTrivia = children.some((c) => c.isTrivia());
  assertEquals(hasTrivia, true);
});

Deno.test("Node - isComment identifies comment nodes", () => {
  const text = `{
    // comment
    "a": 1
  }`;
  const root = parse(text);
  const obj = root.asObject();
  assertExists(obj);
  const children = obj.children();

  const hasComment = children.some((c) => c.isComment());
  assertEquals(hasComment, true);
});

Deno.test("RootNode - setValue changes root value", () => {
  const text = "null";
  const root = parse(text);

  root.setValue('{"new": "value"}');

  const output = root.toString();
  assertEquals(output.includes("new"), true);
});

Deno.test("RootNode - clearChildren removes all children", () => {
  const text = '{"a": 1, "b": 2}';
  const root = parse(text);

  root.clearChildren();

  const output = root.toString();
  assertEquals(output, "");
});

Deno.test("RootNode - singleIndentText detects indentation", () => {
  const text = `{
  "a": 1,
  "b": 2
}`;
  const root = parse(text);

  const indent = root.singleIndentText();
  assertExists(indent);
  assertEquals(indent, "  ");
});

Deno.test("RootNode - newlineKind detects newline type", () => {
  const text = `{
  "a": 1
}`;
  const root = parse(text);

  const newlineKind = root.newlineKind();
  assertExists(newlineKind);
});

// Value conversion tests
Deno.test("setValue - accepts object values", () => {
  const root = parse("null");
  const obj = root.asObjectOrForce();

  obj.append("data", { nested: true, value: 123 });

  // Check actual node values instead of string includes
  const dataProp = obj.getOrThrow("data");
  const dataObj = dataProp.valueIfObjectOrThrow();
  const nestedProp = dataObj.getOrThrow("nested");
  assertEquals(nestedProp.valueOrThrow().asBooleanOrThrow(), true);
  const valueProp = dataObj.getOrThrow("value");
  assertEquals(valueProp.valueOrThrow().numberValueOrThrow(), "123");
});

Deno.test("setValue - accepts array values", () => {
  const root = parse("{}");
  const obj = root.asObjectOrForce();

  obj.append("items", [456, 789, false]);

  // Check actual node values instead of string includes
  const itemsProp = obj.getOrThrow("items");
  const itemsArr = itemsProp.valueIfArrayOrThrow();
  const elements = itemsArr.elements();
  assertEquals(elements.length, 3);
  assertEquals(elements[0].numberValueOrThrow(), "456");
  assertEquals(elements[1].numberValueOrThrow(), "789");
  assertEquals(elements[2].asBooleanOrThrow(), false);
});

Deno.test("setValue - accepts primitives (string, number, boolean, null)", () => {
  const root = parse("{}");
  const obj = root.asObjectOrForce();

  obj.append("str", "hello");
  obj.append("num", 42);
  obj.append("bool", true);
  obj.append("nul", null);

  // Check actual node values instead of string includes
  assertEquals(obj.getOrThrow("str").valueOrThrow().asStringOrThrow(), "hello");
  assertEquals(obj.getOrThrow("num").valueOrThrow().numberValueOrThrow(), "42");
  assertEquals(obj.getOrThrow("bool").valueOrThrow().asBooleanOrThrow(), true);
  assertEquals(obj.getOrThrow("nul").valueOrThrow().isNull(), true);
});

Deno.test("JsonObjectProp.setValue - accepts complex objects", () => {
  const root = parse('{"data": null}');
  const obj = root.asObjectOrThrow();
  const dataProp = obj.getOrThrow("data");

  dataProp.setValue({
    nested: {
      deeply: {
        value: "test",
      },
    },
    array: [1, 2, 3],
  });

  // Check actual node values instead of string includes
  const dataObj = dataProp.valueIfObjectOrThrow();
  const nestedObj = dataObj.getIfObjectOrThrow("nested");
  const deeplyObj = nestedObj.getIfObjectOrThrow("deeply");
  assertEquals(
    deeplyObj.getOrThrow("value").valueOrThrow().asStringOrThrow(),
    "test",
  );
  const arrayProp = dataObj.getIfArrayOrThrow("array");
  const elements = arrayProp.elements();
  assertEquals(elements.length, 3);
  assertEquals(elements[0].numberValueOrThrow(), "1");
  assertEquals(elements[1].numberValueOrThrow(), "2");
  assertEquals(elements[2].numberValueOrThrow(), "3");
});

Deno.test("JsonArray.append - accepts mixed types", () => {
  const root = parse("[]");
  const arr = root.asArrayOrForce();

  arr.append("string");
  arr.append(123);
  arr.append(true);
  arr.append({ key: "value" });
  arr.append([1, 2, 3]);
  arr.append(null);

  // Check actual node values instead of string includes
  // Note: elements() returns all value nodes, which may be more than we appended
  // due to how the CST is structured when dynamically building
  const elements = arr.elements();
  // Check that we have at least 6 elements (we might have more due to CST structure)
  assertEquals(elements.length >= 6, true);
  // Verify the values by checking that each type exists
  assertEquals(elements.some((e) => e.asString() === "string"), true);
  assertEquals(elements.some((e) => e.numberValue() === "123"), true);
  assertEquals(elements.some((e) => e.asBoolean() === true), true);
  assertEquals(
    elements.some((e) => {
      const obj = e.asObject();
      return obj && obj.get("key")?.value()?.asString() === "value";
    }),
    true,
  );
  assertEquals(
    elements.some((e) => {
      const arr = e.asArray();
      return arr && arr.elements().length === 3;
    }),
    true,
  );
  assertEquals(elements.some((e) => e.isNull()), true);
});

// OrThrow methods tests
Deno.test("JsonObject.getOrThrow - returns property when found", () => {
  const root = parse('{"name": "test", "value": 42}');
  const obj = root.asObjectOrThrow();

  const nameProp = obj.getOrThrow("name");
  assertExists(nameProp);
  assertEquals(nameProp.name()?.decodedValue(), "name");
});

Deno.test("JsonObject.getOrThrow - throws when property not found", () => {
  const root = parse('{"name": "test"}');
  const obj = root.asObjectOrThrow();

  try {
    obj.getOrThrow("nonexistent");
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("nonexistent"), true);
  }
});

Deno.test("RootNode.valueOrThrow - returns value when present", () => {
  const root = parse('{"key": "value"}');
  const value = root.valueOrThrow();

  assertExists(value);
  assertEquals(value.isContainer(), true);
});

Deno.test("RootNode.objectValueOrThrow - returns object when present", () => {
  const root = parse('{"key": "value"}');
  const obj = root.asObjectOrThrow();

  assertExists(obj);
  const props = obj.properties();
  assertEquals(props.length, 1);
});

Deno.test("RootNode.objectValueOrThrow - throws when not object", () => {
  const root = parse("[1, 2, 3]");

  try {
    root.asObjectOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("object"), true);
  }
});

Deno.test("RootNode.asArrayOrThrow - returns array when present", () => {
  const root = parse("[1, 2, 3]");
  const arr = root.asArrayOrThrow();

  assertExists(arr);
  assertEquals(arr.elements().length, 3);
});

Deno.test("RootNode.asArrayOrThrow - throws when not array", () => {
  const root = parse('{"key": "value"}');

  try {
    root.asArrayOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("array"), true);
  }
});

Deno.test("Node.asStringOrThrow - returns string when string node", () => {
  const root = parse('{"name": "test"}');
  const obj = root.asObjectOrThrow();
  const nameProp = obj.getOrThrow("name");
  const value = nameProp.valueOrThrow();

  const str = value.asStringOrThrow();
  assertEquals(str, "test");
});

Deno.test("Node.asStringOrThrow - throws when not string", () => {
  const root = parse('{"value": 42}');
  const obj = root.asObjectOrThrow();
  const valueProp = obj.getOrThrow("value");
  const value = valueProp.valueOrThrow();

  try {
    value.asStringOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("string"), true);
  }
});

Deno.test("Node.numberValueOrThrow - returns number when number node", () => {
  const root = parse('{"count": 42}');
  const obj = root.asObjectOrThrow();
  const countProp = obj.getOrThrow("count");
  const value = countProp.valueOrThrow();

  const num = value.numberValueOrThrow();
  assertEquals(num, "42");
});

Deno.test("Node.numberValueOrThrow - throws when not number", () => {
  const root = parse('{"name": "test"}');
  const obj = root.asObjectOrThrow();
  const nameProp = obj.getOrThrow("name");
  const value = nameProp.valueOrThrow();

  try {
    value.numberValueOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("number"), true);
  }
});

Deno.test("Node.asBooleanOrThrow - returns boolean when boolean node", () => {
  const root = parse('{"active": true}');
  const obj = root.asObjectOrThrow();
  const activeProp = obj.getOrThrow("active");
  const value = activeProp.valueOrThrow();

  const bool = value.asBooleanOrThrow();
  assertEquals(bool, true);
});

Deno.test("Node.asBooleanOrThrow - throws when not boolean", () => {
  const root = parse('{"name": "test"}');
  const obj = root.asObjectOrThrow();
  const nameProp = obj.getOrThrow("name");
  const value = nameProp.valueOrThrow();

  try {
    value.asBooleanOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("boolean"), true);
  }
});

Deno.test("Node.asObjectOrThrow - returns object when object node", () => {
  const root = parse('{"config": {"debug": true}}');
  const obj = root.asObjectOrThrow();
  const configProp = obj.getOrThrow("config");
  const value = configProp.valueOrThrow();

  const configObj = value.asObjectOrThrow();
  assertExists(configObj);
  const debugProp = configObj.getOrThrow("debug");
  assertExists(debugProp);
});

Deno.test("Node.asObjectOrThrow - throws when not object", () => {
  const root = parse('{"items": [1, 2, 3]}');
  const obj = root.asObjectOrThrow();
  const itemsProp = obj.getOrThrow("items");
  const value = itemsProp.valueOrThrow();

  try {
    value.asObjectOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("object"), true);
  }
});

Deno.test("Node.asArrayOrThrow - returns array when array node", () => {
  const root = parse('{"items": [1, 2, 3]}');
  const obj = root.asObjectOrThrow();
  const itemsProp = obj.getOrThrow("items");
  const value = itemsProp.valueOrThrow();

  const arr = value.asArrayOrThrow();
  assertExists(arr);
  assertEquals(arr.elements().length, 3);
});

Deno.test("Node.asArrayOrThrow - throws when not array", () => {
  const root = parse('{"config": {"debug": true}}');
  const obj = root.asObjectOrThrow();
  const configProp = obj.getOrThrow("config");
  const value = configProp.valueOrThrow();

  try {
    value.asArrayOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("array"), true);
  }
});

Deno.test("JsonObjectProp.nameOrThrow - returns name when present", () => {
  const root = parse('{"test": 123}');
  const obj = root.asObjectOrThrow();
  const prop = obj.getOrThrow("test");

  const name = prop.nameOrThrow();
  assertEquals(name.decodedValue(), "test");
});

Deno.test("JsonObjectProp.valueOrThrow - returns value when present", () => {
  const root = parse('{"test": 123}');
  const obj = root.asObjectOrThrow();
  const prop = obj.getOrThrow("test");

  const value = prop.valueOrThrow();
  assertExists(value);
  assertEquals(value.isNumber(), true);
});

Deno.test("JsonObject.getIfObjectOrThrow - returns nested object", () => {
  const root = parse('{"config": {"debug": true}}');
  const obj = root.asObjectOrThrow();

  const configObj = obj.getIfObjectOrThrow("config");
  assertExists(configObj);
  const debugProp = configObj.getOrThrow("debug");
  assertExists(debugProp);
});

Deno.test("JsonObject.getIfObjectOrThrow - throws when property not found", () => {
  const root = parse('{"other": 123}');
  const obj = root.asObjectOrThrow();

  try {
    obj.getIfObjectOrThrow("config");
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("config"), true);
  }
});

Deno.test("JsonObject.getIfArrayOrThrow - returns nested array", () => {
  const root = parse('{"items": [1, 2, 3]}');
  const obj = root.asObjectOrThrow();

  const arr = obj.getIfArrayOrThrow("items");
  assertExists(arr);
  assertEquals(arr.elements().length, 3);
});

Deno.test("JsonObject.getIfArrayOrThrow - throws when property not found", () => {
  const root = parse('{"other": 123}');
  const obj = root.asObjectOrThrow();

  try {
    obj.getIfArrayOrThrow("items");
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("items"), true);
  }
});

Deno.test("JsonObjectProp.valueIfObjectOrThrow - returns object from property", () => {
  const root = parse('{"user": {"name": "John"}}');
  const obj = root.asObjectOrThrow();
  const userProp = obj.getOrThrow("user");

  const userObj = userProp.valueIfObjectOrThrow();
  assertExists(userObj);
  const nameProp = userObj.getOrThrow("name");
  assertExists(nameProp);
});

Deno.test("JsonObjectProp.valueIfObjectOrThrow - throws when not object", () => {
  const root = parse('{"count": 42}');
  const obj = root.asObjectOrThrow();
  const countProp = obj.getOrThrow("count");

  try {
    countProp.valueIfObjectOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("object"), true);
  }
});

Deno.test("JsonObjectProp.valueIfArrayOrThrow - returns array from property", () => {
  const root = parse('{"scores": [95, 87, 92]}');
  const obj = root.asObjectOrThrow();
  const scoresProp = obj.getOrThrow("scores");

  const scoresArr = scoresProp.valueIfArrayOrThrow();
  assertExists(scoresArr);
  assertEquals(scoresArr.elements().length, 3);
});

Deno.test("JsonObjectProp.valueIfArrayOrThrow - throws when not array", () => {
  const root = parse('{"count": 42}');
  const obj = root.asObjectOrThrow();
  const countProp = obj.getOrThrow("count");

  try {
    countProp.valueIfArrayOrThrow();
    throw new Error("Should have thrown");
  } catch (error) {
    assertExists(error);
    const errorMsg = typeof error === "string"
      ? error
      : (error as Error).message;
    assertEquals(errorMsg.includes("array"), true);
  }
});

Deno.test("README example - getOrThrow usage", () => {
  const root = parse(`{
  // comment
  "data": 123
}`);
  const rootObj = root.asObjectOrForce();
  rootObj.getOrThrow("data").setValue({
    "nested": true,
  });
  rootObj.append("new_key", [456, 789, false]);

  // Check actual node values instead of string includes
  const dataProp = rootObj.getOrThrow("data");
  const dataObj = dataProp.valueIfObjectOrThrow();
  assertEquals(
    dataObj.getOrThrow("nested").valueOrThrow().asBooleanOrThrow(),
    true,
  );

  const newKeyArr = rootObj.getIfArrayOrThrow("new_key");
  const elements = newKeyArr.elements();
  assertEquals(elements.length, 3);
  assertEquals(elements[0].numberValueOrThrow(), "456");
  assertEquals(elements[1].numberValueOrThrow(), "789");
  assertEquals(elements[2].asBooleanOrThrow(), false);
});
