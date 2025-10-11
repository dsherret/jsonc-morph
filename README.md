# `jsr:@david/jsonc-morph` / `npm:jsonc-morph`

[![JSR](https://jsr.io/badges/@david/jsonc-morph)](https://jsr.io/@david/jsonc-morph)
[![npm version](https://badge.fury.io/js/jsonc-morph.svg)](https://badge.fury.io/js/jsonc-morph)

```
# deno
> deno add @jsr:@david/jsonc-morph
# npm
> npm install jsonc-morph
```

Programmatically edit JSONC files.

This is especially useful for user maintained config files that you need to make
programmatic changes to.

## Example

```ts
import { CstRootNode } from "@david/jsonc-morph";

const root = CstRootNode.parse(`{
  // comment
  "data": 123
}`);
let rootObj = root.objectValueOrSet();
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
