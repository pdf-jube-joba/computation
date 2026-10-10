const assert = require('node:assert/strict');
const path = require('node:path');
const backend = process.argv[2];
const exportsObject = require(path.join(__dirname, backend));
const exportName = backend.includes('.wasm.') ? 'computationOcamlWasm' : 'computationOcamlJs';
function unwrap(text) {
  const result = JSON.parse(text);
  if ('error' in result) throw new Error(result.error);
  return result.ok;
}
const deadline = Date.now() + 10000;
const timer = setInterval(async () => {
  const api = exportsObject[exportName] || globalThis[exportName];
  if (!api) {
    if (Date.now() >= deadline) {
      clearInterval(timer);
      throw new Error(`${backend}: initialization timed out`);
    }
    return;
  }
  clearInterval(timer);
  const a = api.model('example_counter'), b = api.model('example_counter');
  unwrap(a.make('8', '')); unwrap(b.make('2', ''));
  const saved = unwrap(a.snapshot());
  assert.deepEqual(unwrap(a.step('inc')), {kind: 'continue', output: 'inc'});
  assert.deepEqual(unwrap(b.snapshot()), {count: 2});
  assert.deepEqual(unwrap(a.step('inc')), {kind: 'halt', output: 'End'});
  assert.ok(JSON.parse(a.step('inc')).error);
  unwrap(a.restore(JSON.stringify(saved)));
  assert.deepEqual(unwrap(a.snapshot()), {count: 8});
  assert.ok(JSON.parse(a.restore('{')).error);
  assert.deepEqual(unwrap(a.snapshot()), {count: 8});
  assert.equal(unwrap(a.render())[0].rows[0].cells[1].text, '8');
  const lambda = api.model('lambda_calculus');
  unwrap(lambda.make('(\\x. x) (\\y. y)', ''));
  const term = unwrap(lambda.snapshot());
  assert.deepEqual(unwrap(lambda.step('0')), {kind: 'halt', output: '\\y. y'});
  unwrap(lambda.restore(JSON.stringify(term)));
  assert.deepEqual(unwrap(lambda.snapshot()), term);
  assert.ok(JSON.parse(lambda.step('1')).error);
  assert.deepEqual(unwrap(lambda.step('0')), {kind: 'halt', output: '\\y. y'});
  const {createAdapter} = await import('./model_adapter.mjs');
  const mounted = createAdapter(api, 'example_counter');
  mounted.make('8', '');
  const snapshot = mounted.snapshot();
  assert.deepEqual(JSON.parse(mounted.step('inc')), {kind: 'continue', routput: 'inc'});
  // Rendering a saved snapshot does not restore or change the active state.
  assert.equal(JSON.parse(mounted.render(snapshot))[0].rows[0].cells[1].text, '8');
  assert.deepEqual(JSON.parse(mounted.snapshot()), {count: 9});
  assert.deepEqual(JSON.parse(mounted.step('inc')), {kind: 'halt', foutput: 'End'});
  assert.equal(JSON.parse(mounted.render(snapshot))[0].rows[0].cells[1].text, '8');
  mounted.restore(snapshot);
  assert.throws(() => mounted.step('bad'), /Expected inc or dec/);
  assert.equal(mounted.snapshot(), snapshot);
  const mountedLambda = createAdapter(api, 'lambda_calculus');
  mountedLambda.make('(\\x. x) (\\y. y)', '');
  assert.deepEqual(JSON.parse(mountedLambda.step('0')), {kind: 'halt', foutput: '\\y. y'});
  console.log(`${backend}: JS API tests passed`);
}, 20);
