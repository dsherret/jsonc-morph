// ex. scripts/build_npm.ts
import { build, emptyDir } from "@deno/dnt";

await emptyDir("./npm");

await build({
  entryPoints: ["./mod.ts"],
  outDir: "./npm",
  shims: {
    deno: {
      test: true,
    },
  },
  compilerOptions: {
    target: "Latest",
  },
  package: {
    name: "jsonc-morph",
    version: Deno.args[0],
    description: "Programmatic code changes of JSONC.",
    license: "MIT",
    repository: {
      type: "git",
      url: "git+https://github.com/dsherret/jsonc-morph.git",
    },
    bugs: {
      url: "https://github.com/dsherret/jsonc-morph/issues",
    },
  },
  postBuild() {
    // steps to run after building and before running the tests
    Deno.copyFileSync("LICENSE", "npm/LICENSE");
    Deno.copyFileSync("README.md", "npm/README.md");

    // dnt infers declarations from the js and loses the wasm-bindgen types
    const dts = Deno.readTextFileSync("lib/rs_lib.d.ts");
    for (const dir of ["esm", "script"]) {
      Deno.writeTextFileSync(
        `npm/${dir}/lib/rs_lib.d.ts`,
        `/// <reference lib="esnext.disposable" />\n${dts}`,
      );
      Deno.removeSync(`npm/${dir}/lib/rs_lib.internal.d.ts`);
    }
  },
});
