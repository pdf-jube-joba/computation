import {build} from "esbuild";
import {cp, mkdir} from "node:fs/promises";
import path from "node:path";
import {fileURLToPath} from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const srcDir = path.join(__dirname, "src");
const outDir = parseOutDir(process.argv.slice(2));
const generatedDir = path.join(outDir, "generated");
const katexDistDir = path.join(__dirname, "node_modules", "katex", "dist");
const katexOutDir = path.join(outDir, "vendor", "katex");
const macrosSource = path.join(__dirname, "..", "..", "docs", "macros.txt");
const macrosOutPath = path.join(outDir, "macros.txt");

await mkdir(outDir, {recursive: true});
await mkdir(generatedDir, {recursive: true});
await mkdir(katexOutDir, {recursive: true});

if (path.resolve(macrosSource) !== path.resolve(macrosOutPath)) {
  await cp(macrosSource, macrosOutPath);
}
await cp(path.join(katexDistDir, "katex.min.css"), path.join(katexOutDir, "katex.min.css"));
await cp(path.join(katexDistDir, "fonts"), path.join(katexOutDir, "fonts"), {recursive: true});

await build({
  entryPoints: [path.join(srcDir, "markdown_viewer.js")],
  bundle: true,
  format: "esm",
  platform: "browser",
  target: "es2022",
  outfile: path.join(generatedDir, "markdown_viewer.js"),
  sourcemap: true,
  logLevel: "info",
});

function parseOutDir(args) {
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] !== "--out-dir") {
      continue;
    }

    const value = args[index + 1];
    if (!value) {
      throw new Error("missing value for --out-dir");
    }

    return path.resolve(value);
  }

  throw new Error("missing required argument: --out-dir <path>");
}
