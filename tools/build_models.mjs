import {cp, mkdir, writeFile, rm} from 'node:fs/promises';
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
run(['build', ...profile, 'web/browser.bc.js', 'web/browser.bc.wasm.js', 'cli/main.exe']);
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
const built = path.join(repoRoot, '_build/default/web');
for (const file of ['browser.bc.js', 'browser.bc.wasm.js']) await cp(path.join(built, file), path.join(outputDir, file));
await cp(path.join(built, 'browser.bc.wasm.assets'), path.join(outputDir, 'browser.bc.wasm.assets'), {recursive: true});
for (const file of ['runtime.mjs', 'model_adapter.mjs']) await cp(path.join(repoRoot, 'web/runtime', file), path.join(outputDir, file));
const published = [];
for (const entry of entries) {
  for (const backend of ['wasm', 'js']) {
    const name = entry.name + (backend === 'js' ? '_js' : '');
    const factory = entry.kind === 'model' ? 'createModel' : 'createCompiler';
    const exports = entry.kind === 'model' ? 'make, step, snapshot, restore, render'
      : 'compileCode, encodeAinput, encodeRinput, decodeRoutput, decodeFoutput';
    await writeFile(path.join(outputDir, `${name}.js`), `import {${factory}} from './runtime.mjs';
const instance = await ${factory}(${JSON.stringify(entry.name)}, '${backend}');
export const {${exports}} = instance;
`);
    published.push({...entry, name, backend});
  }
}
await writeFile(path.join(outputDir, 'models.json'), JSON.stringify({entries: published, loaders: {
  wasm: 'browser.bc.wasm.js', js: 'browser.bc.js',
}}, null, 2) + '\n');
console.log(`Built ${entries.length} models/compilers (JS + Wasm): ${outputDir}`);
