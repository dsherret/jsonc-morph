# `jsr:@david/jsonc-morph` / `npm:jsonc-morph`

[![JSR](https://jsr.io/badges/@david/jsonc-morph)](https://jsr.io/@david/jsonc-morph)
[![npm version](https://badge.fury.io/js/jsonc-morph.svg)](https://badge.fury.io/js/jsonc-morph)

Programmatically edit JSONC in JavaScript.

This is especially useful for making programmatic changes to JSON config files.
It's not recommended for very large files as this is using
[jsonc-parser](https://github.com/dprint/jsonc-parser/) via Wasm under the hood.

## Install

Deno:

```
deno add @jsr:@david/jsonc-morph
```

Or with npm:

```
npm install jsonc-morph
```

## Example

```ts
import { parse } from "@david/jsonc-morph";

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
assertEquals(root.toString(), `{
  // 1
  "data" /* 2 */: {
    "nested": true
  }, // 3
  "new_key": [456, 789, false]
} // 4`);
```
