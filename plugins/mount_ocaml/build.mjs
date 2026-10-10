import {cp, mkdir, writeFile} from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

const repoRoot = fileURLToPath(new URL('../../', import.meta.url));
const ocamlRoot = path.join(repoRoot, 'ocaml');
let outputDir = process.env.WORKSPACE_FS_OUTPUT_DIRECTORY || path.join(repoRoot, 'ocaml_bundle');
const args = process.argv.slice(2);
if (args.length) {
  if (args.length !== 2 || args[0] !== '--out-dir') throw new Error('Usage: build.mjs [--out-dir PATH]');
  outputDir = path.resolve(args[1]);
}
const result = spawnSync('opam', ['exec', '--', 'dune', 'build',
  'web/browser.bc.js', 'web/browser.bc.wasm.js'], {cwd: ocamlRoot, stdio: 'inherit'});
if (result.error) throw result.error;
if (result.status !== 0) throw new Error(`OCaml build failed: ${result.status}`);

await mkdir(outputDir, {recursive: true});
const built = path.join(ocamlRoot, '_build/default/web');
// Distinct export names let both backends coexist on the same Markdown page.
for (const file of ['browser.bc.js', 'browser.bc.wasm.js']) {
  await cp(path.join(built, file), path.join(outputDir, file));
}
await cp(path.join(built, 'browser.bc.wasm.assets'), path.join(outputDir, 'browser.bc.wasm.assets'), {recursive: true});
for (const file of ['runtime.mjs', 'model_adapter.mjs']) {
  await cp(path.join(ocamlRoot, 'web', file), path.join(outputDir, file));
}
for (const [moduleName, model] of [
  ['ocaml_example_counter', 'counter'], ['ocaml_lambda_calculus', 'lambda'],
]) {
  for (const backend of ['wasm', 'js']) {
    const fileName = `${moduleName}${backend === 'js' ? '_js' : ''}.js`;
    await writeFile(path.join(outputDir, fileName), `import {createModel} from './runtime.mjs';
const model = await createModel('${model}', '${backend}');
export const {make, step, snapshot, restore, render} = model;
`);
  }
}
console.log(`OCaml browser bundles: ${outputDir}`);
