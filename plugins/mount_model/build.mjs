import {copyFile, mkdir, readdir, readFile, rm, stat} from "node:fs/promises";
import {existsSync} from "node:fs";
import path from "node:path";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const SCRIPT_DIR = __dirname;
const REPO_ROOT = path.resolve(SCRIPT_DIR, "..", "..");
const MODELS_DIR = path.join(REPO_ROOT, "models");
const TARGET_DIR = path.join(REPO_ROOT, "target", "wasm32-unknown-unknown");
const STATIC_FILES = ["renderer.js", "script.js", "style.css", "mount.js"];

const args = parseArgs(process.argv.slice(2));
const outputDir = process.env.WORKSPACE_FS_OUTPUT_DIRECTORY;
if (!outputDir) {
  throw new Error("WORKSPACE_FS_OUTPUT_DIRECTORY is required");
}

ensureTools();

if (args.clean && existsSync(outputDir)) {
  await rm(outputDir, {recursive: true, force: true});
}

await mkdir(outputDir, {recursive: true});
await copyStaticAssets(outputDir);

const profile = args.release ? "release" : "debug";
let packages = await findPackages(MODELS_DIR);

if (args.packages.size > 0) {
  packages = packages.filter(([name]) => args.packages.has(name));
  const found = new Set(packages.map(([name]) => name));
  const missing = Array.from(args.packages).filter(name => !found.has(name)).sort();
  if (missing.length > 0) {
    throw new Error(`unknown package(s): ${missing.join(", ")}`);
  }
}

if (packages.length === 0) {
  console.log("[warn] no packages to build");
  process.exit(0);
}

for (const [packageName, crateDir] of packages) {
  const binNames = await resolveBinNames(crateDir, packageName);
  if (binNames.length === 0) {
    console.log(`[skip] ${packageName}: no bin targets`);
    continue;
  }

  for (const binName of binNames) {
    const outNames = [binName];
    if (binNames.length === 1 && packageName !== binName) {
      outNames.push(packageName);
    }
    console.log(`[build] package=${packageName} bin=${binName} out=${outNames.join(",")}`);
    await buildBin({
      packageName,
      binName,
      profile,
      release: args.release,
      outNames,
      outputDir,
    });
  }
}

function parseArgs(argv) {
  const packages = new Set();
  let release = false;
  let clean = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--package") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("missing value for --package");
      }
      packages.add(value);
      index += 1;
      continue;
    }
    if (arg === "--release") {
      release = true;
      continue;
    }
    if (arg === "--clean") {
      clean = true;
      continue;
    }
    throw new Error(`unknown argument: ${arg}`);
  }

  return {packages, release, clean};
}

function ensureTools() {
  const required = ["cargo", "wasm-tools"];
  const missing = required.filter(name => !findCommand(name));
  if (missing.length > 0) {
    throw new Error(`missing required tools: ${missing.join(", ")}`);
  }
  if (!existsSync(localBin("jco"))) {
    throw new Error("missing local jco; run `npm install` in plugin/mount_model");
  }
}

function findCommand(name) {
  const result = spawnSync("sh", ["-lc", `command -v ${name}`], {
    cwd: REPO_ROOT,
    stdio: "ignore",
  });
  return result.status === 0;
}

function localBin(name) {
  const binName = process.platform === "win32" ? `${name}.cmd` : name;
  return path.join(SCRIPT_DIR, "node_modules", ".bin", binName);
}

function run(cmd, args, cwd = REPO_ROOT) {
  console.log("[run]", [cmd, ...args].join(" "));
  const result = spawnSync(cmd, args, {
    cwd,
    stdio: "inherit",
    env: process.env,
  });
  if (result.status !== 0) {
    throw new Error(`command failed (${result.status ?? "signal"}): ${[cmd, ...args].join(" ")}`);
  }
}

async function copyStaticAssets(outputDirPath) {
  for (const filename of STATIC_FILES) {
    await copyFile(path.join(SCRIPT_DIR, filename), path.join(outputDirPath, filename));
  }
}

async function findPackages(rootDir) {
  if (!existsSync(rootDir)) {
    return [];
  }

  const entries = await readdir(rootDir, {withFileTypes: true});
  const packages = [];
  for (const entry of entries.toSorted((a, b) => a.name.localeCompare(b.name))) {
    if (!entry.isDirectory()) {
      continue;
    }
    const cargoToml = path.join(rootDir, entry.name, "Cargo.toml");
    if (!existsSync(cargoToml)) {
      continue;
    }
    const name = await readPackageName(cargoToml);
    if (name) {
      packages.push([name, path.join(rootDir, entry.name)]);
    }
  }
  return packages;
}

async function readPackageName(cargoTomlPath) {
  let inPackage = false;
  const text = await readFile(cargoTomlPath, "utf8");
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }
    if (line === "[package]") {
      inPackage = true;
      continue;
    }
    if (inPackage && line.startsWith("[")) {
      break;
    }
    if (inPackage && line.startsWith("name")) {
      const [, value] = line.split("=", 2);
      return value.trim().replace(/^"|"$/g, "");
    }
  }
  return null;
}

async function readBins(cargoTomlPath) {
  const bins = [];
  let inBin = false;
  let current = {};
  const text = await readFile(cargoTomlPath, "utf8");
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }
    if (line === "[[bin]]") {
      if (inBin && Object.keys(current).length > 0) {
        bins.push(current);
      }
      current = {};
      inBin = true;
      continue;
    }
    if (line.startsWith("[")) {
      if (inBin && Object.keys(current).length > 0) {
        bins.push(current);
      }
      inBin = false;
      continue;
    }
    if (inBin && line.includes("=")) {
      const [key, value] = line.split("=", 2);
      const normalizedKey = key.trim();
      if (normalizedKey === "name" || normalizedKey === "path") {
        current[normalizedKey] = value.trim().replace(/^"|"$/g, "");
      }
    }
  }
  if (inBin && Object.keys(current).length > 0) {
    bins.push(current);
  }
  return bins;
}

async function resolveBinNames(crateDir, packageName) {
  const binNames = [];
  const seen = new Set();
  const push = name => {
    if (!seen.has(name)) {
      seen.add(name);
      binNames.push(name);
    }
  };

  if (existsSync(path.join(crateDir, "src", "main.rs"))) {
    push(packageName);
  }

  const cargoTomlPath = path.join(crateDir, "Cargo.toml");
  if (!existsSync(cargoTomlPath)) {
    return binNames;
  }

  for (const entry of await readBins(cargoTomlPath)) {
    const name = entry.name;
    const entryPath = entry.path;
    if (!name) {
      continue;
    }
    if (entryPath) {
      if (existsSync(path.join(crateDir, entryPath))) {
        push(name);
      }
      continue;
    }
    if (existsSync(path.join(crateDir, "src", "bin", `${name}.rs`))) {
      push(name);
    }
  }

  return binNames;
}

async function isUpToDate(outputPath, inputPaths) {
  if (!existsSync(outputPath)) {
    return false;
  }
  const outputStat = await stat(outputPath);
  for (const inputPath of inputPaths) {
    if (!existsSync(inputPath)) {
      return false;
    }
    const inputStat = await stat(inputPath);
    if (inputStat.mtimeMs > outputStat.mtimeMs) {
      return false;
    }
  }
  return true;
}

async function buildBin({packageName, binName, profile, release, outNames, outputDir: outputDirPath}) {
  const buildArgs = [
    "build",
    "--package",
    packageName,
    "--target",
    "wasm32-unknown-unknown",
    "--bin",
    binName,
  ];
  if (release) {
    buildArgs.push("--release");
  }
  run("cargo", buildArgs, REPO_ROOT);

  const coreWasm = path.join(TARGET_DIR, profile, `${binName}.wasm`);
  if (!existsSync(coreWasm)) {
    throw new Error(`missing wasm output: ${coreWasm}`);
  }

  await mkdir(outputDirPath, {recursive: true});
  for (const outName of outNames) {
    const componentWasm = path.join(outputDirPath, `${outName}.component.wasm`);
    const jcoJs = path.join(outputDirPath, `${outName}.js`);
    const jcoCore = path.join(outputDirPath, `${outName}.core.wasm`);

    if (!(await isUpToDate(componentWasm, [coreWasm]))) {
      run("wasm-tools", ["component", "new", coreWasm, "-o", componentWasm], REPO_ROOT);
    } else {
      console.log(`[skip] component up-to-date: ${path.basename(componentWasm)}`);
    }

    if (!((await isUpToDate(jcoJs, [componentWasm])) && (await isUpToDate(jcoCore, [componentWasm])))) {
      run(localBin("jco"), [
        "transpile",
        componentWasm,
        "-o",
        outputDirPath,
        "--name",
        outName,
        "--no-typescript",
        "-q",
      ], REPO_ROOT);
    } else {
      console.log(`[skip] jco output up-to-date: ${outName}`);
    }
  }
}
