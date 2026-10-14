// Exercise the same model/codec contracts through the actual browser Wasm API.
// Usage: node tests/wasm_smoke.cjs /path/to/bundle
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const bundle = path.resolve(process.argv[2] || 'wasm_bundle');
const readFixture = name => JSON.parse(fs.readFileSync(path.join(__dirname, name), 'utf8'));
const models = readFixture('model_fixtures.json');
const compilers = readFixture('compiler_fixtures.json');
// wasm_of_ocaml's Node loader resolves its assets relative to the entry script.
require.main.filename = path.join(bundle, 'browser.bc.wasm.js');
require(path.join(bundle, 'browser.bc.wasm.js'));

function unwrap(text) {
  const response = JSON.parse(text);
  if ('error' in response) throw new Error(response.error);
  return response.ok;
}

async function main() {
  for (let i = 0; i < 250 && !globalThis.computationOcamlWasm; i++) {
    await new Promise(resolve => setTimeout(resolve, 20));
  }
  const api = globalThis.computationOcamlWasm;
  assert.ok(api, 'Wasm API did not initialize');
  for (const fixture of models) {
    const model = api.model(fixture.name);
    assert.equal(unwrap(model.make(fixture.code, fixture.ainput)), null);
    if (fixture.initial_snapshot) {
      assert.deepEqual(unwrap(model.snapshot()), fixture.initial_snapshot, fixture.name);
    }
    for (let i = 0; i < fixture.steps; i++) {
      const snapshot = model.snapshot();
      const saved = unwrap(snapshot);
      const rendered = unwrap(model.render());
      assert.deepEqual(unwrap(model.renderSnapshot(JSON.stringify(saved))), rendered);
      assert.equal(unwrap(model.restore(JSON.stringify(saved))), null);
      assert.deepEqual(unwrap(model.snapshot()), saved);
      const result = unwrap(model.step(fixture.rinput));
      if (result.kind === 'halt') {
        assert.equal(i, fixture.steps - 1, fixture.name);
        assert.equal(result.output, fixture.final, fixture.name);
      } else {
        assert.equal(result.kind, 'continue');
        assert.equal(result.output, fixture.outputs[i], fixture.name);
        if (i === fixture.steps - 1) assert.equal(fixture.final, null);
      }
    }
  }
  for (const fixture of compilers) {
    const compiler = api.compiler(fixture.name);
    const model = api.model(fixture.target);
    const code = unwrap(compiler.compileCode(fixture.code));
    const ainput = unwrap(compiler.encodeAinput(fixture.ainput));
    const rinput = unwrap(compiler.encodeRinput(fixture.rinput));
    unwrap(model.make(code, ainput));
    let halted = false;
    for (let i = 0; i < 20000; i++) {
      const result = unwrap(model.step(rinput));
      if (result.kind === 'halt') {
        assert.equal(unwrap(compiler.decodeFoutput(result.output)), fixture.expected, fixture.name);
        halted = true;
        break;
      }
      unwrap(compiler.decodeRoutput(result.output));
    }
    assert.ok(halted, `${fixture.name}: step limit exceeded`);
  }
  const entries = JSON.parse(api.manifest());
  const names = new Set([...models, ...compilers].map(f => f.name));
  assert.ok(entries.every(entry => names.has(entry.name)), 'untested registry entry');
  console.log(`Wasm passed ${models.length} model fixtures and ${compilers.length} compiler paths.`);
}
main().catch(error => { console.error(error); process.exitCode = 1; });
