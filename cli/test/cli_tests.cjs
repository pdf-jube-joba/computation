const assert = require('node:assert/strict');
const path = require('node:path');
const {spawnSync} = require('node:child_process');
const executable = path.resolve(process.argv[2]);
function run(args, input = '', expectedStatus = 0) {
  const result = spawnSync(executable, args, {input, encoding: 'utf8'});
  assert.ifError(result.error);
  assert.equal(result.status, expectedStatus, result.stderr);
  return result;
}
const manifest = JSON.parse(run(['list', '--json']).stdout);
assert.ok(manifest.some(entry => entry.name === 'lambda_calculus' && entry.kind === 'model'));
const model = ['model', 'example_counter'];
const created = run([...model, 'create', '--code', '8']).stdout;
assert.deepEqual(JSON.parse(created), {count: 8});
const continued = run([...model, 'step', '--rinput', '\\inc'], created);
assert.deepEqual(JSON.parse(continued.stdout), {count: 9});
assert.equal(JSON.parse(continued.stderr), 'inc');
const halted = run([...model, 'step', '--rinput', '\\inc'], continued.stdout);
assert.equal(halted.stdout, '');
assert.equal(JSON.parse(halted.stderr), 'End');
const failed = run([...model, 'step', '--rinput', 'bad'], created, 1);
assert.equal(failed.stdout, '');
assert.match(failed.stderr, /Expected (?:.*inc or .*dec|keyword)/);
assert.equal(JSON.parse(run([...model, 'render'], created).stdout)[0].rows[0].cells[1].text, '8');
const code = run(['compiler', 'compile-code', 'recursive_function-goto_lang', '--code', '\\SUCC']).stdout;
const input = run(['compiler', 'compile-ainput', 'recursive_function-goto_lang', '--ainput', '(2)']).stdout;
let state = run(['model', 'goto_lang', 'create', '--code', code, '--ainput', input]).stdout;
let output;
for (let fuel = 100; fuel > 0; fuel--) {
  const next = run(['model', 'goto_lang', 'step', '--rinput', ''], state);
  if (!next.stdout) { output = JSON.parse(next.stderr); break; }
  state = next.stdout;
}
assert.equal(run(['compiler', 'decode-foutput', 'recursive_function-goto_lang', '--foutput', output]).stdout.trim(), '3');
const repl = run(['repl', 'model', 'example_counter', '--code', '8', '--ainput', ''], 'bad\n\\inc\n\\inc\n');
assert.match(repl.stderr, /Expected (?:.*inc or .*dec|keyword)/);
assert.match(repl.stdout, /"count":9/);
assert.match(repl.stdout, /halted/);
const multiline = run(['repl', 'model', 'lambda_calculus'], ':begin\n(\\x.\nx) (\\y. y)\n:end\n\n0\n');
assert.match(multiline.stdout, /halted/);
console.log('Native CLI pipe, compiler, error recovery, and multiline REPL tests passed');
