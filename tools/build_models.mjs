import {cp, mkdir, readFile, writeFile, rm, rmdir} from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const repoRoot = fileURLToPath(new URL('../', import.meta.url));
const options = {clean: false, release: false, packages: new Set(), models: new Set()};
let outputDir = process.env.WORKSPACE_FS_OUTPUT_DIRECTORY || path.join(repoRoot, 'wasm_bundle');
const args = process.argv.slice(2);
for (let i = 0; i < args.length; i++) {
  const arg = args[i];
  if (arg === '--clean') options.clean = true;
  else if (arg === '--release') options.release = true;
  else if (['--out-dir', '--package', '--model'].includes(arg)) {
    if (!args[i + 1]) throw new Error(`Missing value for ${arg}`);
    const value = args[++i];
    if (arg === '--out-dir') outputDir = path.resolve(value);
    else options[arg === '--package' ? 'packages' : 'models'].add(value);
  } else throw new Error(`Unknown argument: ${arg}`);
}
function run(args, capture = false) {
  const result = spawnSync('opam', ['exec', '--', 'dune', ...args], {
    cwd: repoRoot, stdio: capture ? ['ignore', 'pipe', 'inherit'] : 'inherit', encoding: 'utf8',
  });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Dune failed (${result.status}): ${args.join(' ')}`);
  return result.stdout;
}
const profile = options.release ? ['--profile', 'release'] : [];
run(['build', ...profile, 'web/browser.bc.wasm.js', 'cli/main.exe']);
let entries = JSON.parse(run(['exec', ...profile, 'cli/main.exe', '--', 'list', '--json'], true));
if (options.models.size || options.packages.size) {
  const selected = entries.filter(entry => options.models.has(entry.name)
    || options.packages.has(entry.library || entry.name));
  for (const name of options.models) if (!entries.some(entry => entry.name === name)) throw new Error(`Unknown model: ${name}`);
  for (const name of options.packages) if (!entries.some(entry => (entry.library || entry.name) === name)) throw new Error(`Unknown library: ${name}`);
  entries = selected;
}
if (options.clean) {
  const relative = path.relative(path.resolve(outputDir), repoRoot);
  if (!relative.startsWith('..') && !path.isAbsolute(relative)) throw new Error('Refusing to clean repository root or its parent');
  await rm(outputDir, {recursive: true, force: true});
}
await mkdir(outputDir, {recursive: true});
// Remove previously published modules, including the old flat JS/Wasm pairs.
// Limit removal to paths this script generates, even for a malformed manifest.
for (const manifestName of ['models.json', 'manifest.json']) {
  let previous;
  try {
    previous = JSON.parse(await readFile(path.join(outputDir, manifestName), 'utf8'));
  } catch (error) {
    if (error.code === 'ENOENT') continue;
    throw error;
  }
  for (const entry of previous.entries) {
    const file = manifestName === 'models.json' ? `${entry.name}.js` : entry.module;
    if (/^(?:(?:models|compiles)\/)?[\w-]+\.js$/.test(file)) {
      await rm(path.join(outputDir, file), {force: true});
    }
  }
}
for (const file of ['models.json', 'browser.bc.js', 'browser.bc.wasm.assets']) {
  await rm(path.join(outputDir, file), {recursive: true, force: true});
}
for (const directory of ['models', 'compiles']) {
  // Remove obsolete adapter directories only when empty.
  try {
    await rmdir(path.join(outputDir, directory));
  } catch (error) {
    if (!['ENOENT', 'ENOTEMPTY'].includes(error.code)) throw error;
  }
}
const built = path.join(repoRoot, '_build/default/web');
await cp(path.join(built, 'browser.bc.wasm.js'), path.join(outputDir, 'browser.bc.wasm.js'));
await cp(path.join(built, 'browser.bc.wasm.assets'), path.join(outputDir, 'browser.bc.wasm.assets'), {recursive: true});
for (const file of ['runtime.mjs', 'model_adapter.mjs']) await cp(path.join(repoRoot, 'web/runtime', file), path.join(outputDir, file));
const published = entries.map(entry => ({...entry, backend: 'wasm'}));
await writeFile(path.join(outputDir, 'manifest.json'), JSON.stringify({entries: published, runtime: 'runtime.mjs', loaders: {
  wasm: 'browser.bc.wasm.js',
}}, null, 2) + '\n');
console.log(`Built ${entries.length} models/compilers (Wasm): ${outputDir}`);
