# jsonc-morph

[![JSR](https://jsr.io/badges/@david/jsonc-morph)](https://jsr.io/@david/jsonc-morph)
[![npm version](https://badge.fury.io/js/jsonc-morph.svg)](https://badge.fury.io/js/jsonc-morph)

Parse and edit JSONC in JavaScript.

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

## Sorting

Objects and arrays can be reordered, and what was written with a member travels
with it: the comments and blank lines above it, and a comment written after it
on the same line.

```ts
import { parse } from "@david/jsonc-morph";

const root = parse(`{
  // the version to publish
  "version": "1.0.0",
  "name": "example"
}`);

root.asObjectOrThrow().sortProperties();

assertEquals(
  root.toString(),
  `{
  "name": "example",
  // the version to publish
  "version": "1.0.0"
}`,
);
```

`sortProperties` sorts by name when called with no argument. Pass a comparator
to order properties some other way, such as a conventional field order:

```ts
const order = ["name", "version", "description", "dependencies"];

root.asObjectOrThrow().sortProperties((a, b) =>
  order.indexOf(a.decodedName() ?? "") - order.indexOf(b.decodedName() ?? "")
);
```

`sortElements` does the same for arrays, sorting by each element's text when
called with no argument:

```ts
root.asArrayOrThrow().sortElements();
root.asArrayOrThrow().sortElements((a, b) =>
  Number(a.toString()) - Number(b.toString())
);
```

Both sorts are stable, so members that compare equal keep the order they were
written in, and both keep whichever of a trailing comma or none the container
was written with. If the comparator throws, the container is left exactly as it
was and the error is rethrown.

### What travels with a member

A comment above a member travels with that member, which is right for a comment
describing it and wrong for one heading a whole group. Pass `pinCommentHeaders`
to leave a comment that has a blank line above it where it was written:

```ts
const root = parse(`{
  "prop": 1,

  // section
  "prop2": 2,
  "prop1": 1
}`);

root.asObjectOrThrow().sortProperties(undefined, { pinCommentHeaders: true });

assertEquals(
  root.toString(),
  `{
  "prop": 1,

  // section
  "prop1": 1,
  "prop2": 2
}`,
);
```

The blank line is what tells the two kinds apart, so a comment written flush
against its member still travels with it either way.

Pass a function when a block is partly a heading and partly a note about the
member beneath it. It is handed the member and the comments written above it,
and returns how many of them, counting from the top, stay where they are:

```ts
// the first comment heads the group, the rest belong to the member
root.asObjectOrThrow().sortProperties(undefined, {
  pinCommentHeaders: (member) => member.hasBlankLineBefore() ? 1 : 0,
});
```

### Sorting within groups

`pinCommentHeaders` pins the comment but members still sort past the blank line.
Pass `withinGroups` to sort each run of members between blank lines on its own
instead, so that no member crosses one:

```ts
const root = parse(`[
  30,

  // small ones
  20,
  10
]`);

root.asArrayOrThrow().sortElements(undefined, { withinGroups: true });

assertEquals(
  root.toString(),
  `[
  30,

  // small ones
  10,
  20
]`,
);
```

## Options and strict parsing

By default, `parse` and `parseToValue` allow more than just JSONC (comments,
trailing commas, single-quoted strings, hexadecimal numbers, etc.). You can
customize this behaviour by passing options:

```ts
import { parse, parseToValue } from "@david/jsonc-morph";

// disable specific extensions
const root = parse(text, {
  allowComments: false, // reject // and /* */ comments
  allowTrailingCommas: false, // reject trailing commas in arrays/objects
  allowSingleQuotedStrings: false, // reject 'single quoted' strings
  allowHexadecimalNumbers: false, // reject 0xFF style numbers
  allowUnaryPlusNumbers: false, // reject +42 style numbers
  allowMissingCommas: false, // reject missing commas between elements
  allowLooseObjectPropertyNames: false, // reject unquoted property names
});

// parseToValue accepts the same options
const value = parseToValue(text, { allowComments: false });
```

For strict JSON parsing (only allow JSON), use `parseStrict` or
`parseToValueStrict`:

```ts
import { parseStrict, parseToValueStrict } from "@david/jsonc-morph";

// all extensions disabled by default
const root = parseStrict('{"name": "test"}');

// selectively enable specific extensions
const rootWithComments = parseStrict(text, { allowComments: true });

// same for `parseToValueStrict`
const value = parseToValueStrict('{"name": "test"}');
```
