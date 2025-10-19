/**
 * Post-build script to replace 'any' types with 'JsonValue' in generated TypeScript definitions
 */

const dtsPath = "./lib/rs_lib.d.ts";

// Read the generated .d.ts file
let content = await Deno.readTextFile(dtsPath);

// Replace 'any' types with 'JsonValue' for specific parameters
// Be careful to only replace in function signatures, not in other contexts

const replacements: Array<[RegExp, string]> = [
  // RootNode.setValue
  [/setValue\(root_value: any\)/g, "setValue(root_value: JsonValue)"],

  // JsonObject methods
  [
    /append\(key: string, value: any\)/g,
    "append(key: string, value: JsonValue)",
  ],
  [
    /insert\(index: number, key: string, value: any\)/g,
    "insert(index: number, key: string, value: JsonValue)",
  ],

  // JsonArray methods
  [/append\(value: any\)/g, "append(value: JsonValue)"],
  [
    /insert\(index: number, value: any\)/g,
    "insert(index: number, value: JsonValue)",
  ],

  // ObjectProp.setValue
  [/setValue\(value: any\)/g, "setValue(value: JsonValue)"],

  // All replaceWith methods
  [/replaceWith\(replacement: any\)/g, "replaceWith(replacement: JsonValue)"],
  [/replaceWith\(value: any\)/g, "replaceWith(value: JsonValue)"],
  [
    /replaceWith\(key: string, replacement: any\)/g,
    "replaceWith(key: string, replacement: JsonValue)",
  ],

  // parseToValue and toValue methods
  [/parseToValue\([^)]*\): any/g, (match) => match.replace(": any", ": JsonValue")],
  [/toValue\(\): any/g, "toValue(): JsonValue"],

  [/newlineKind\(\): string/g, 'newlineKind(): "\\n" | "\\r\\n"'],
];

// Apply all replacements
for (const [pattern, replacement] of replacements) {
  content = content.replace(pattern, replacement);
}

if (content.includes(": any")) {
  console.log(content);
  throw new Error("Found any type after build!");
}

// Write the modified content back
await Deno.writeTextFile(dtsPath, content);

console.log("✓ Fixed TypeScript types in rs_lib.d.ts");
