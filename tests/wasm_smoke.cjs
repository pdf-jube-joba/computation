// Exercise the shared runtime factories through the actual browser Wasm API.
// Usage: node tests/wasm_smoke.cjs /path/to/bundle
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {pathToFileURL} = require('node:url');

const bundle = path.resolve(process.argv[2] || 'wasm_bundle');
const readFixture = name => JSON.parse(fs.readFileSync(path.join(__dirname, name), 'utf8'));
const models = readFixture('model_fixtures.json');
const compilers = readFixture('compiler_fixtures.json');
// wasm_of_ocaml's Node loader resolves its assets relative to the entry script.
require.main.filename = path.join(bundle, 'browser.bc.wasm.js');
require(path.join(bundle, 'browser.bc.wasm.js'));

async function main() {
  for (let i = 0; i < 250 && !globalThis.computationOcamlWasm; i++) {
    await new Promise(resolve => setTimeout(resolve, 20));
  }
  const api = globalThis.computationOcamlWasm;
  assert.ok(api, 'Wasm API did not initialize');
  const manifest = JSON.parse(fs.readFileSync(path.join(bundle, 'manifest.json'), 'utf8'));
  const {createModel, createCompiler} = await import(pathToFileURL(path.join(bundle, manifest.runtime)));
  for (const fixture of models) {
    const model = await createModel(fixture.name);
    model.make(fixture.code, fixture.ainput);
    if (fixture.initial_snapshot) {
      assert.deepEqual(JSON.parse(model.snapshot()), fixture.initial_snapshot, fixture.name);
    }
    for (let i = 0; i < fixture.steps; i++) {
      const snapshot = model.snapshot();
      const saved = JSON.parse(snapshot);
      const rendered = JSON.parse(model.render(snapshot));
      model.restore(JSON.stringify(saved));
      assert.deepEqual(JSON.parse(model.snapshot()), saved);
      assert.deepEqual(JSON.parse(model.render(model.snapshot())), rendered);
      const result = JSON.parse(model.step(fixture.rinput));
      if (result.kind === 'halt') {
        assert.equal(i, fixture.steps - 1, fixture.name);
        assert.equal(result.foutput, fixture.final, fixture.name);
      } else {
        assert.equal(result.kind, 'continue');
        assert.equal(result.routput, fixture.outputs[i], fixture.name);
        if (i === fixture.steps - 1) assert.equal(fixture.final, null);
      }
    }
  }
  for (const fixture of compilers) {
    const compiler = await createCompiler(fixture.name);
    const model = await createModel(fixture.target);
    const code = compiler.compileCode(fixture.code);
    const ainput = compiler.encodeAinput(fixture.ainput);
    const rinput = compiler.encodeRinput(fixture.rinput);
    model.make(code, ainput);
    let halted = false;
    for (let i = 0; i < 20000; i++) {
      const result = JSON.parse(model.step(rinput));
      if (result.kind === 'halt') {
        assert.equal(compiler.decodeFoutput(result.foutput), fixture.expected, fixture.name);
        halted = true;
        break;
      }
      compiler.decodeRoutput(result.routput);
    }
    assert.ok(halted, `${fixture.name}: step limit exceeded`);
  }
  // Repeated mounts/reloads must get separate state despite sharing one runtime import.
  const [first, second] = await Promise.all([
    createModel('example_counter'), createModel('example_counter'),
  ]);
  first.make('3', '');
  second.make('3', '');
  const initial = second.snapshot();
  first.step('\\inc');
  assert.notEqual(first.snapshot(), initial);
  assert.equal(second.snapshot(), initial, 'model sessions must be independent');
  const reloaded = await createModel('example_counter');
  reloaded.make('5', '');
  assert.equal(second.snapshot(), initial, 'creating a session must not reset another');
  const entries = JSON.parse(api.manifest());
  const names = new Set([...models, ...compilers].map(f => f.name));
  assert.ok(entries.every(entry => names.has(entry.name)), 'untested registry entry');
  console.log(`Wasm passed ${models.length} model fixtures and ${compilers.length} compiler paths.`);
}
main().catch(error => { console.error(error); process.exitCode = 1; });
