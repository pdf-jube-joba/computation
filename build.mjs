#!/usr/bin/env node
import {existsSync} from 'node:fs';
import {copyFile, mkdir, readFile, writeFile} from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('./', import.meta.url));
const moduleName = 'computation/models';
const expectedBindgenVersion = 'wit-bindgen-cli 0.63.0';
const componentHeader = Buffer.from([0, 97, 115, 109, 13, 0, 1, 0]);

async function main() {
  const entries = JSON.parse(await readFile(path.join(root, 'components.json'), 'utf8'));
  const {names, outputDir, list} = parseArguments(process.argv.slice(2));
  const selected = selectComponents(entries, names);

  if (list) {
    for (const entry of selected) console.log(entry.name);
    return;
  }

  await mkdir(outputDir, {recursive: true});
  checkTools();
  for (const entry of selected) await buildComponent(entry, outputDir);
  await writeManifest(entries, outputDir);
}

function parseArguments(args) {
  const names = [];
  let outputDir = path.join(root, 'build/assets');
  let list = false;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--list') {
      list = true;
    } else if (arg === '--model' || arg === '--out-dir') {
      const value = args[++i];
      if (!value) throw new Error(`Missing value for ${arg}`);
      if (arg === '--model') names.push(value);
      else outputDir = path.resolve(value);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return {names, outputDir, list};
}

function selectComponents(entries, names) {
  if (!names.length) return entries;
  return names.map(name => {
    const entry = entries.find(entry => entry.name === name);
    if (!entry) throw new Error(`Unknown MoonBit component: ${name}`);
    return entry;
  });
}

function checkTools() {
  const version = run('wit-bindgen', ['--version'], {capture: true}).trim();
  if (version !== expectedBindgenVersion) {
    throw new Error(`Expected ${expectedBindgenVersion}, found ${version}`);
  }
  run('jco', ['--version'], {capture: true});
}

async function buildComponent(entry, outputDir) {
  const packagePath = `build/components/${entry.name}`;
  const componentDir = path.join(root, packagePath);
  await generateBindings(entry, packagePath, componentDir);
  const coreWasm = buildMoonBit(packagePath);

  const embeddedWasm = path.join(componentDir, 'embedded.wasm');
  const componentWasm = path.join(outputDir, `${entry.name}.wasm`);
  run('wasm-tools', [
    'component', 'embed', 'computation.wit', coreWasm,
    '--world', entry.kind, '--encoding', 'utf16', '--output', embeddedWasm,
  ]);
  run('wasm-tools', ['component', 'new', embeddedWasm, '--output', componentWasm]);
  run('wasm-tools', ['validate', componentWasm]);
  run('jco', [
    'transpile', componentWasm, '--out-dir', outputDir, '--name', entry.name,
    '--instantiation', 'async', '--no-typescript',
  ], {capture: true});
  console.log(`Built ${entry.name}: ${componentWasm}`);
}

async function generateBindings(entry, packagePath, componentDir) {
  await mkdir(componentDir, {recursive: true});
  run('wit-bindgen', [
    'moonbit', 'computation.wit', '--world', entry.kind, '--ignore-module-file',
    '--project-name', `${moduleName}/${packagePath}`, '--out-dir', componentDir,
  ], {capture: true});

  const adapterDir = path.join(componentDir, 'gen/world', entry.kind);
  await patchPostReturn(path.join(adapterDir, 'ffi.mbt'));
  await writeAdapter(entry, adapterDir);
}

async function patchPostReturn(ffiPath) {
  // wit-bindgen 0.63.0 allocates each result record but only frees its payload
  // in cabi_post_*. Reclaim that record too; otherwise long training runs leak
  // one allocation per call. Keep the generator pinned until upstream fixes it.
  const ffi = await readFile(ffiPath, 'utf8');
  let fixed = 0;
  const postReturn = /(pub fn wasmExport\w+PostReturn\(p0 : Int\) -> Unit \{[\s\S]*?)(\n\})/g;
  const patched = ffi.replace(postReturn, (_, body, end) => {
    fixed++;
    return `${body}\n      mbt_ffi_free(p0)\n${end}`;
  });
  if (fixed === 0) {
    throw new Error(`Unexpected ${expectedBindgenVersion} post-return code (${fixed} functions)`);
  }
  await writeFile(ffiPath, patched);
}

async function writeAdapter(entry, adapterDir) {
  const imports = `import {
  "${moduleName}/${entry.package}" @target,
  "${moduleName}/runtime",
  "${moduleName}/utils",
}
warnings = "-44-27"
`;
  await writeFile(path.join(adapterDir, 'moon.pkg'), imports);
  const targetFactory = entry.factory || entry.kind;
  const sessionType = entry.kind === 'model' ? 'Session' : 'CompilerSession';
  const sessionFactory = entry.kind === 'model' ? 'session' : 'compiler_session';
  await copyFile(
    path.join(root, 'adapters', `${entry.kind}.mbt`),
    path.join(adapterDir, 'implementation.mbt'),
  );
  // Only session initialization depends on the selected component.
  await writeFile(path.join(adapterDir, 'session.mbt'),
    `// Generated by build.mjs.\nlet machine : @runtime.${sessionType} = @runtime.${sessionFactory}(@target.${targetFactory}())\n`);
}

function buildMoonBit(packagePath) {
  const buildLog = run('moon', [
    'build', '--target', 'wasm', '--release', '--deny-warn', '--output-json', `${packagePath}/gen`,
  ], {capture: true});
  // The standard path also works with older moon versions that don't emit artifact messages.
  const fallback = path.join(root, '_build/wasm/release/build', packagePath, 'gen/gen.wasm');
  for (const line of buildLog.split('\n')) {
    let message;
    try { message = JSON.parse(line); }
    catch { continue; }
    if (message?.$message_type === 'artifact' || message?.type === 'artifact') {
      return message.path || fallback;
    }
  }
  return fallback;
}

async function writeManifest(entries, outputDir) {
  // A partial build must not hide components already built into this directory.
  const manifest = [];
  for (const entry of entries) {
    const file = path.join(outputDir, `${entry.name}.wasm`);
    if (!existsSync(file)) continue;
    const bytes = await readFile(file);
    if (bytes.subarray(0, componentHeader.length).equals(componentHeader)) manifest.push(entry);
  }
  await writeFile(path.join(outputDir, 'components.json'), JSON.stringify(manifest, null, 2) + '\n');
}

function run(command, args, {capture = false} = {}) {
  const result = spawnSync(command, args, {
    cwd: root, encoding: 'utf8',
    stdio: capture ? ['ignore', 'pipe', 'pipe'] : ['ignore', 'inherit', 'inherit'],
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} failed (${result.status})\n${result.stdout || ''}\n${result.stderr || ''}`);
  }
  return result.stdout;
}

await main();
