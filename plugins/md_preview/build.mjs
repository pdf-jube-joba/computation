import {build} from "esbuild";
import {cp, mkdir} from "node:fs/promises";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {generateLinkIndex} from "./build_index.mjs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const srcDir = path.join(__dirname, "src");
const args = parseArgs(process.argv.slice(2));
const outDir = args.outDir;
const repositoryRoot = args.repositoryRoot;
const katexDistDir = path.join(__dirname, "node_modules", "katex", "dist");
const katexOutDir = path.join(outDir, "vendor", "katex");
const macrosSource = path.join(__dirname, "..", "..", "docs", "macros.txt");
const macrosOutPath = path.join(outDir, "macros.txt");

await mkdir(outDir, {recursive: true});
await mkdir(katexOutDir, {recursive: true});

if (path.resolve(macrosSource) !== path.resolve(macrosOutPath)) {
  await cp(macrosSource, macrosOutPath);
}
await cp(path.join(katexDistDir, "katex.min.css"), path.join(katexOutDir, "katex.min.css"));
await cp(path.join(katexDistDir, "fonts"), path.join(katexOutDir, "fonts"), {recursive: true});
await generateLinkIndex({outDir, repositoryRoot});

await build({
  entryPoints: [path.join(srcDir, "markdown_viewer.js")],
  bundle: true,
  format: "esm",
  platform: "browser",
  target: "es2022",
  outfile: path.join(outDir, "markdown_viewer.js"),
  define: {
    "__WASM_MOUNT_URL__": JSON.stringify(args.wasmMountUrl),
  },
  sourcemap: false,
  logLevel: "info",
});

function parseArgs(argv) {
  let outDir = null;
  let wasmMountUrl = "/wasm_bundle/";
  let repositoryRoot = process.cwd();

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--out-dir") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("missing value for --out-dir");
      }
      outDir = path.resolve(value);
      index += 1;
      continue;
    }
    if (arg === "--wasm-mount-url") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("missing value for --wasm-mount-url");
      }
      wasmMountUrl = normalizeMountUrl(value);
      index += 1;
      continue;
    }
    if (arg === "--repository-root") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("missing value for --repository-root");
      }
      repositoryRoot = path.resolve(value);
      index += 1;
      continue;
    }
    throw new Error(`unknown argument: ${arg}`);
  }

  if (!outDir) {
    throw new Error("missing required argument: --out-dir <path>");
  }

  return {outDir, wasmMountUrl, repositoryRoot};
}

function normalizeMountUrl(value) {
  if (!value.startsWith("/") || !value.endsWith("/")) {
    throw new Error(`invalid mount url: ${value}`);
  }
  return value;
}
