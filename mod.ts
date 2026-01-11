export {
  BooleanLit,
  JsonArray,
  JsonObject,
  type JsonValue,
  Node,
  NullKeyword,
  NumberLit,
  ObjectProp,
  ObjectPropName,
  parse,
  type ParseOptions,
  parseToValue,
  RootNode,
  StringLit,
  WordLit,
} from "./lib/rs_lib.js";

import {
  type JsonValue,
  parse,
  type ParseOptions,
  parseToValue,
  type RootNode,
} from "./lib/rs_lib.js";

/**
 * Options for strict JSON parsing (all JSONC extensions disabled by default).
 * @deprecated Use `ParseOptions` instead - this is an alias for backwards compatibility.
 */
export type ParseStrictOptions = ParseOptions;

const STRICT_DEFAULTS: Required<ParseOptions> = {
  allowComments: false,
  allowTrailingCommas: false,
  allowLooseObjectPropertyNames: false,
  allowMissingCommas: false,
  allowSingleQuotedStrings: false,
  allowHexadecimalNumbers: false,
  allowUnaryPlusNumbers: false,
};

/**
 * Parses a strict JSON string into a concrete syntax tree.
 * By default, all JSONC extensions are disabled (no comments, no trailing commas, etc.).
 * You can selectively enable extensions by setting options to `true`.
 * @param text - The JSON text to parse
 * @param options - Optional parsing options (all default to `false`)
 * @returns The root node of the parsed CST
 */
export function parseStrict(
  text: string,
  options?: ParseOptions,
): RootNode {
  return parse(text, { ...STRICT_DEFAULTS, ...options });
}

/**
 * Parses a strict JSON string directly to a JavaScript value.
 * By default, all JSONC extensions are disabled (no comments, no trailing commas, etc.).
 * You can selectively enable extensions by setting options to `true`.
 * @param text - The JSON text to parse
 * @param options - Optional parsing options (all default to `false`)
 * @returns The plain JavaScript value (object, array, string, number, boolean, or null)
 * @throws If the text cannot be parsed or converted
 */
export function parseToValueStrict(
  text: string,
  options?: ParseOptions,
): JsonValue {
  return parseToValue(text, { ...STRICT_DEFAULTS, ...options });
}
