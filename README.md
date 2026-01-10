# `jsr:@david/jsonc-morph` / `npm:jsonc-morph`

[![JSR](https://jsr.io/badges/@david/jsonc-morph)](https://jsr.io/@david/jsonc-morph)
[![npm version](https://badge.fury.io/js/jsonc-morph.svg)](https://badge.fury.io/js/jsonc-morph)

Programmatically edit JSONC in JavaScript.

This is especially useful for making programmatic changes to JSON config files.

## Install

Deno:

```
deno add jsr:@david/jsonc-morph
```

Or with npm:

```
npm install jsonc-morph
```

## Example

```ts
import { parse, parseToValue } from "@david/jsonc-morph";

// parse directly to a plain JavaScript value
const config = parseToValue(`{
  // database settings
  "host": "localhost",
  "port": 5432
}`);

console.log(config.host); // "localhost"
console.log(config.port); // 5432

// parse to a CST for programmatic editing
const root = parse(`{
  // 1
  "data" /* 2 */: 123 // 3
} // 4`);

// get the root object
const rootObj = root.asObjectOrThrow();

// set its "data" property to have a new value
rootObj.getOrThrow("data").setValue({
  "nested": true,
});

// append a new key
rootObj.append("new_key", [456, 789, false]);

// inspect the output
assertEquals(
  root.toString(),
  `{
  // 1
  "data" /* 2 */: {
    "nested": true
  }, // 3
  "new_key": [456, 789, false]
} // 4`,
);
```

## Options and strict parsing

By default, `parse` and `parseToValue` allow more than just JSONC (comments,
trailing commas, single-quoted strings, hexadecimal numbers, etc.). You can
customize this behaviour by passing options:

```ts
import { parse, parseToValue } from "@david/jsonc-morph";

// Disable specific extensions
const root = parse(text, {
  allowComments: false, // Reject // and /* */ comments
  allowTrailingCommas: false, // Reject trailing commas in arrays/objects
  allowSingleQuotedStrings: false, // Reject 'single quoted' strings
  allowHexadecimalNumbers: false, // Reject 0xFF style numbers
  allowUnaryPlusNumbers: false, // Reject +42 style numbers
  allowMissingCommas: false, // Reject missing commas between elements
  allowLooseObjectPropertyNames: false, // Reject unquoted property names
});

// parseToValue accepts the same options
const value = parseToValue(text, { allowComments: false });
```

For strict JSON parsing (only allow JSON), use `parseStrict` or
`parseToValueStrict`:

```ts
import { parseStrict, parseToValueStrict } from "@david/jsonc-morph";

// All extensions disabled by default
const root = parseStrict('{"name": "test"}');

// Selectively enable specific extensions
const rootWithComments = parseStrict(text, { allowComments: true });

// Same for parseToValueStrict
const value = parseToValueStrict('{"name": "test"}');
```
