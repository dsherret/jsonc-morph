/**
 * Type checks a consumer of the built npm package from both ESM and CJS to
 * ensure the published declarations keep their types.
 */

await main();

async function main() {
  let failed = false;
  for (const packageType of ["module", "commonjs"]) {
    const dir = await Deno.makeTempDir();
    try {
      await setUpConsumer(dir, packageType);
      const success = await typeCheck(dir);
      console.log(`${success ? "✓" : "✗"} npm types (${packageType})`);
      failed ||= !success;
    } finally {
      await Deno.remove(dir, { recursive: true });
    }
  }
  if (failed) {
    Deno.exit(1);
  }
}

async function setUpConsumer(dir: string, packageType: string) {
  await Deno.writeTextFile(
    `${dir}/package.json`,
    JSON.stringify({ private: true, type: packageType }),
  );
  await Deno.writeTextFile(`${dir}/check.ts`, consumerSource());
  const packageDir = `${dir}/node_modules/jsonc-morph`;
  await Deno.mkdir(packageDir, { recursive: true });
  await Deno.copyFile("npm/package.json", `${packageDir}/package.json`);
  for (const subDir of ["esm", "script"]) {
    await copyDir(`npm/${subDir}`, `${packageDir}/${subDir}`);
  }
}

async function typeCheck(cwd: string) {
  const { success } = await new Deno.Command(Deno.execPath(), {
    args: [
      "run",
      "-A",
      "--no-lock",
      "npm:typescript@5.9/tsc",
      "--noEmit",
      "--strict",
      "--skipLibCheck",
      "false",
      "--target",
      "es2022",
      "--module",
      "node16",
      "--moduleResolution",
      "node16",
      "check.ts",
    ],
    cwd,
  }).spawn().status;
  return success;
}

function consumerSource() {
  return `import { type JsonObject, type ObjectProp, parse, parseStrict } from "jsonc-morph";

type IsAny<T> = 0 extends (1 & T) ? true : false;
function notAny<T>(_value: IsAny<T> extends true ? never : T) {}

const root = parse('{ "a": 1 }');
notAny(root);
notAny(parseStrict("{}"));

const obj: JsonObject = root.asObjectOrThrow();
notAny(obj);
notAny(obj.get("a"));
notAny(obj.properties());
notAny(obj.getIfObjectOrThrow("a"));
notAny(obj.insert(0, "b", 1));

const prop: ObjectProp = obj.getOrThrow("a");
notAny(prop);
notAny(prop.name());

declare const unknownValue: unknown;
// @ts-expect-error unknown is not a JsonValue
prop.setValue(unknownValue);
`;
}

async function copyDir(from: string, to: string) {
  await Deno.mkdir(to, { recursive: true });
  for await (const entry of Deno.readDir(from)) {
    const source = `${from}/${entry.name}`;
    const target = `${to}/${entry.name}`;
    if (entry.isDirectory) {
      await copyDir(source, target);
    } else {
      await Deno.copyFile(source, target);
    }
  }
}
