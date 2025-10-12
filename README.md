# `jsr:@david/jsonc-morph` / `npm:jsonc-morph`

[![JSR](https://jsr.io/badges/@david/jsonc-morph)](https://jsr.io/@david/jsonc-morph)
[![npm version](https://badge.fury.io/js/jsonc-morph.svg)](https://badge.fury.io/js/jsonc-morph)

Programmatically edit JSONC in JavaScript.

This is especially useful for config files that you need to make programmatic
changes to. It's not recommended for very large files as this is using
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
  // comment
  "data": 123
}`);
let rootObj = root.asObjectOrThrow();
rootObj.getOrThrow("data").setValue({
  "nested": true,
});
rootObj.append("new_key", [456, 789, false]);
const output = root.toString();
// {
//   // comment
//   "data": {
//     "nested": true
//   },
//   "new_key": [456, 789, false]
// }
console.log(output);
```
