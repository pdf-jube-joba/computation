/* Documented browser fixtures, independent of registry/build ownership.
 *   node web/fixtures.cjs --inventory
 *   node web/fixtures.cjs --refresh       # update only the JSON inventory
 *   node web/fixtures.cjs                 # test both compiled backends
 *   node web/fixtures.cjs _build/default/web/browser.bc.js
 * Build browser.bc.js/browser.bc.wasm.js before running. Missing public names
 * always fail; an unavailable endpoint is never an exclusion.
 */
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {fork} = require('node:child_process');

function findRoot() {
  for (const start of [__dirname, process.cwd()]) {
    for (let dir = start; ; dir = path.dirname(dir)) {
      if (fs.existsSync(path.join(dir, 'test.md')) &&
          fs.existsSync(path.join(dir, 'models')) && fs.existsSync(path.join(dir, 'docs'))) return dir;
      if (path.dirname(dir) === dir) break;
    }
  }
  throw new Error('Cannot find repository models/, docs/, and test.md');
}
const root = findRoot();
const dataPath = path.join(root, 'tests/model_fixtures.json');

function markdownFiles(dir) {
  return fs.readdirSync(dir, {withFileTypes: true}).flatMap(entry => {
    const filename = path.join(dir, entry.name);
    // Dune may stage source dependencies as symlinks. Dirent.isFile() alone
    // would silently omit those documented samples from the inventory.
    const kind = entry.isSymbolicLink() ? fs.statSync(filename) : entry;
    if (kind.isDirectory()) return markdownFiles(filename);
    return kind.isFile() && /\.md$/i.test(entry.name) ? [filename] : [];
  });
}

function attribute(tag, name) {
  const match = tag.match(new RegExp(`\\b${name}\\s*=\\s*(?:"([^"]*)"|'([^']*)'|([^\\s>]+))`, 'i'));
  return match ? match[1] ?? match[2] ?? match[3] : undefined;
}

function inventory() {
  const files = [...markdownFiles(path.join(root, 'models')),
    ...markdownFiles(path.join(root, 'docs')), path.join(root, 'test.md')].sort();
  const fixtures = [];
  for (const filename of files) {
    const text = fs.readFileSync(filename, 'utf8');
    const source = path.relative(root, filename).split(path.sep).join('/');
    let ordinal = 0;
    for (const opening of text.matchAll(/<div\b[^>]*>/gi)) {
      const model = attribute(opening[0], 'data-model');
      if (model === undefined) continue;
      const start = opening.index + opening[0].length;
      const tags = /<\/?div\b[^>]*>/gi;
      tags.lastIndex = start;
      let depth = 1, closing;
      while ((closing = tags.exec(text))) {
        depth += /^<\//.test(closing[0]) ? -1 : 1;
        if (depth === 0) break;
      }
      assert.equal(depth, 0, `${source}: unclosed data-model div`);
      const body = text.slice(start, closing.index);
      const templates = {};
      for (const template of body.matchAll(/<template\b([^>]*)>([\s\S]*?)<\/template\s*>/gi)) {
        for (const field of ['code', 'ainput', 'rinput']) {
          if (!new RegExp(`\\bdata-default-${field}(?:\\s|=|$)`, 'i').test(template[1])) continue;
          assert.ok(!(field in templates), `${source}: duplicate default-${field}`);
          templates[field] = template[2].trim();
        }
      }
      fixtures.push({id: `${source}#${++ordinal}`, source,
        line: text.slice(0, opening.index).split('\n').length, model, templates});
    }
  }
  return fixtures;
}

function describe(fixtures) {
  const names = [...new Set(fixtures.map(fixture => fixture.model))].sort();
  const defaults = fixtures.filter(fixture => Object.keys(fixture.templates).length > 0).length;
  console.log(`${fixtures.length} documented mounts; ${names.length} public names; ${defaults} with defaults; ${fixtures.length - defaults} empty mounts`);
  for (const name of names) {
    const sources = fixtures.filter(fixture => fixture.model === name)
      .map(fixture => `${fixture.source}:${fixture.line}${Object.keys(fixture.templates).length ? '' : ' (no defaults)'}`);
    console.log(`  ${name}: ${sources.join(', ')}`);
  }
}

function envelope(text, context) {
  let result;
  try { result = JSON.parse(text); }
  catch (error) { throw new Error(`${context}: invalid response JSON: ${error.message}`); }
  assert.ok(result && typeof result === 'object', `${context}: expected an API envelope`);
  assert.equal(('ok' in result) !== ('error' in result), true, `${context}: expected exactly one of ok/error`);
  return result;
}
function call(object, method, args, context) {
  assert.equal(typeof object[method], 'function', `${context}: missing ${method} API method`);
  const result = envelope(object[method](...args), `${context}.${method}`);
  if ('error' in result) throw new Error(`${context}.${method}: ${result.error}`);
  return result.ok;
}
function render(value, context) {
  assert.ok(Array.isArray(value), `${context}: render must be a block array`);
  for (const block of value) {
    assert.ok(block && typeof block === 'object' && typeof block.kind === 'string', `${context}: malformed render block`);
  }
}

function machine(api, name, inputs, context) {
  const session = api.model(name);
  call(session, 'make', [inputs.code, inputs.ainput], context);
  const saved = call(session, 'snapshot', [], context);
  const rendered = call(session, 'render', [], context);
  render(rendered, context);
  const restored = api.model(name);
  call(restored, 'restore', [JSON.stringify(saved)], context);
  assert.deepEqual(call(restored, 'snapshot', [], context), saved, `${context}: snapshot/restore changed state`);
  assert.deepEqual(call(restored, 'render', [], context), rendered, `${context}: restored render changed`);
  const step = call(session, 'step', [inputs.rinput], context);
  assert.ok(step && ['continue', 'halt'].includes(step.kind), `${context}: invalid step result`);
  assert.ok('output' in step, `${context}: missing step output`);
  assert.deepEqual(call(restored, 'step', [inputs.rinput], context), step, `${context}: restore changed step behavior`);
  let next = null;
  if (step.kind === 'continue') {
    next = call(session, 'snapshot', [], context);
    assert.deepEqual(call(restored, 'snapshot', [], context), next, `${context}: restored next state differs`);
    render(call(session, 'render', [], context), context);
    call(restored, 'restore', [JSON.stringify(next)], context);
    assert.deepEqual(call(restored, 'snapshot', [], context), next, `${context}: next snapshot cannot be restored`);
  }
  call(session, 'restore', [JSON.stringify(saved)], context);
  assert.deepEqual(call(session, 'step', [inputs.rinput], context), step, `${context}: replay differs`);
  return {saved, rendered, step, next};
}

function inputsFor(fixture, data) {
  return {code: fixture.templates.code ?? '', ainput: fixture.templates.ainput ?? '',
    rinput: fixture.templates.rinput ?? data.stepInputs[fixture.model] ?? ''};
}

function canonicalName(name, data) {
  const seen = new Set();
  while (data.aliases?.[name]) {
    assert.ok(!seen.has(name), `Alias cycle involving ${name}`);
    seen.add(name);
    const alias = data.aliases[name];
    assert.equal(typeof alias.canonical, 'string', `${name}: alias needs a canonical endpoint`);
    assert.ok(typeof alias.evidence === 'string' && alias.evidence.length > 0,
      `${name}: alias needs evidence that both entries use the same implementation`);
    name = alias.canonical;
  }
  return name;
}

function exercise(api, fixture, data, entries) {
  const context = `${fixture.id} (${fixture.model})`;
  const entry = entries.get(fixture.model);
  assert.ok(entry, `${context}: public endpoint missing from manifest`);
  if (data.expectedKinds[fixture.model]) {
    assert.equal(entry.kind, data.expectedKinds[fixture.model], `${context}: public endpoint changed kind`);
  }
  const endpoint = entry.kind === 'model' ? api.model(fixture.model) : api.compiler(fixture.model);
  assert.ok(endpoint, `${context}: endpoint lookup failed`);
  const inputs = inputsFor(fixture, data);
  // Exclusions match the exact legacy code, so a corrected doc is automatically
  // exercised positively after --refresh. No parser failure creates an exclusion.
  const exclusion = data.exclusions.find(excluded => excluded.id === fixture.id && excluded.code === inputs.code);
  if (exclusion) {
    assert.ok(exclusion.reason && exclusion.evidence.length > 0, `${context}: exclusion needs original implementation evidence`);
    assert.equal(entry.kind, 'model', `${context}: invalid exclusion kind`);
    const response = envelope(endpoint.make(inputs.code, inputs.ainput), `${context}.make`);
    assert.ok('error' in response, `${context}: legacy invalid sample unexpectedly accepted; review exclusion`);
    assert.match(response.error, new RegExp(exclusion.errorPattern, 'i'), `${context}: unexpected rejection`);
    return {kind: 'legacy-invalid', reason: exclusion.reason};
  }
  if (Object.keys(fixture.templates).length === 0) {
    // An empty mount supplies no sample. Require its name and endpoint; the
    // separate supplemental fixture provides runnable inputs for this name.
    assert.ok(data.supplemental.some(extra => extra.model === fixture.model), `${context}: empty mount lacks runnable supplemental coverage`);
    return {kind: 'empty-mount', endpointKind: entry.kind};
  }
  if (entry.kind === 'model') return {kind: 'model', result: machine(api, fixture.model, inputs, context)};
  const mapping = data.compilerTargets[fixture.model];
  assert.ok(mapping, `${context}: compiler source/target mapping missing`);
  assert.equal(entries.get(mapping.source)?.kind, 'model', `${context}: compiler source model missing`);
  assert.equal(entries.get(mapping.target)?.kind, 'model', `${context}: compiler target model missing`);
  const source = machine(api, mapping.source, inputs, `${context} source`);
  const targetInputs = {
    code: call(endpoint, 'compileCode', [inputs.code], context),
    ainput: call(endpoint, 'encodeAinput', [inputs.ainput], context),
    rinput: call(endpoint, 'encodeRinput', [inputs.rinput], context),
  };
  for (const [name, text] of Object.entries(targetInputs)) assert.equal(typeof text, 'string', `${context}: ${name} is not printable text`);
  const target = machine(api, mapping.target, targetInputs, `${context} target`);
  const decoder = target.step.kind === 'continue' ? 'decodeRoutput' : 'decodeFoutput';
  // The legacy Lambda unit codec returns JSON null. Its TextCodec value is
  // empty text, matching the browser adapter's unit-output convention.
  const output = target.step.output === null ? '' : target.step.output;
  assert.equal(typeof output, 'string', `${context}: target output is not printable text`);
  const decoded = call(endpoint, decoder, [output], context);
  assert.equal(typeof decoded, 'string', `${context}: decoded output is not printable text`);
  return {kind: 'compiler', source, targetInputs, target, decoded};
}

async function loadBackend(filename) {
  const resolved = path.isAbsolute(filename) ? filename : path.resolve(process.cwd(), filename);
  const wasm = filename.includes('.wasm.');
  const mainFilename = require.main.filename;
  try {
    // wasm_of_ocaml loads assets relative to require.main.filename, rather than
    // its own bundle filename. Keep that directory correct for source-side
    // invocations as well as Dune's copied script next to the built bundle.
    if (wasm) require.main.filename = resolved;
    const exported = require(resolved);
    const exportName = wasm ? 'computationOcamlWasm' : 'computationOcamlJs';
    const deadline = Date.now() + 10000;
    return await new Promise((resolve, reject) => {
      const timer = setInterval(() => {
        const api = exported[exportName] || globalThis[exportName];
        if (api) { clearInterval(timer); resolve(api); }
        else if (Date.now() >= deadline) { clearInterval(timer); reject(new Error(`${filename}: initialization timed out`)); }
      }, 20);
    });
  } finally { require.main.filename = mainFilename; }
}

async function runBackend(filename, data) {
  const api = await loadBackend(filename);
  const manifest = JSON.parse(api.manifest());
  assert.ok(Array.isArray(manifest), `${filename}: manifest must be an array`);
  const entries = new Map();
  for (const entry of manifest) {
    assert.equal(typeof entry.name, 'string', `${filename}: malformed manifest entry`);
    assert.ok(['model', 'compiler'].includes(entry.kind), `${filename}: invalid manifest kind`);
    assert.ok(!entries.has(entry.name), `${filename}: duplicate public name ${entry.name}`);
    entries.set(entry.name, entry);
  }
  const failures = [], results = [], coverage = new Set();
  for (const name of Object.keys(data.expectedKinds)) {
    if (!entries.has(name)) failures.push(`${name}: expected public endpoint missing from manifest`);
  }
  // Even entries without a Markdown fixture must expose their promised API.
  for (const entry of manifest) {
    try {
      assert.equal(entry.kind, data.expectedKinds[entry.name], `${entry.name}: missing or changed expected endpoint kind`);
      const canonical = canonicalName(entry.name, data);
      const implementation = entries.get(canonical);
      assert.ok(implementation, `${entry.name}: alias points to missing endpoint ${canonical}`);
      assert.equal(implementation.kind, entry.kind, `${entry.name}: alias changes endpoint kind`);
      if (entry.library !== undefined && implementation.library !== undefined) {
        assert.equal(entry.library, implementation.library, `${entry.name}: alias points to a different library`);
      }
      const endpoint = entry.kind === 'model' ? api.model(entry.name) : api.compiler(entry.name);
      const methods = entry.kind === 'model' ? ['make', 'step', 'snapshot', 'restore', 'render']
        : ['compileCode', 'encodeAinput', 'encodeRinput', 'decodeRoutput', 'decodeFoutput'];
      for (const method of methods) assert.equal(typeof endpoint[method], 'function', `${entry.name}: missing ${method}`);
    } catch (error) { failures.push(`manifest ${entry.name}: ${error.message}`); }
  }
  for (const fixture of [...data.fixtures, ...data.supplemental]) {
    try {
      const result = exercise(api, fixture, data, entries);
      results.push({id: fixture.id, result});
      if (result.kind === 'model' || result.kind === 'compiler') {
        coverage.add(canonicalName(fixture.model, data));
      }
      if (result.kind === 'compiler') {
        const mapping = data.compilerTargets[fixture.model];
        coverage.add(canonicalName(mapping.source, data));
        coverage.add(canonicalName(mapping.target, data));
      }
    }
    catch (error) { failures.push(`${fixture.id}: ${error.message}`); }
  }
  const implementations = new Set([...entries.keys()].map(name => canonicalName(name, data)));
  for (const name of implementations) {
    if (!coverage.has(name)) failures.push(`${name}: registered implementation has no successful runnable fixture`);
  }
  console.log(`${filename}: ${results.length}/${data.fixtures.length + data.supplemental.length} fixtures passed; ${coverage.size}/${implementations.size} implementations executed; ${entries.size} manifest endpoints checked`);
  for (const failure of failures) console.error(`  FAIL ${failure}`);
  return {manifest, results, coverage: [...coverage].sort(), failures};
}

function runChild(filename) {
  return new Promise((resolve, reject) => {
    // The OCaml runtimes install global handlers. Separate processes keep the
    // JS and Wasm runtimes independent; only their JSON reports are compared.
    const child = fork(__filename, ['--worker', filename], {stdio: ['ignore', 'inherit', 'inherit', 'ipc']});
    let report;
    child.on('message', message => {
      report = message.report;
      child.disconnect();
    });
    child.on('error', reject);
    child.on('exit', (code, signal) => {
      if (report) resolve(report);
      else reject(new Error(`${filename}: fixture subprocess exited ${signal || code} without a report`));
    });
  });
}

async function main() {
  const worker = process.argv.includes('--worker');
  const args = process.argv.slice(2).filter(arg => arg !== '--worker');
  const current = inventory();
  if (args.includes('--inventory')) { describe(current); return; }
  const data = JSON.parse(fs.readFileSync(dataPath, 'utf8'));
  assert.equal(data.version, 1, 'Unsupported fixture data version');
  if (args.includes('--refresh')) {
    data.fixtures = current;
    fs.writeFileSync(dataPath, JSON.stringify(data, null, 2) + '\n');
    describe(current);
    console.log('Updated tests/model_fixtures.json; exclusion evidence and supplemental inputs preserved');
    return;
  }
  assert.deepEqual(data.fixtures, current, 'Markdown fixtures changed; run node web/fixtures.cjs --refresh and review the JSON');
  assert.ok(!args.some(arg => arg.startsWith('--')), 'Unknown option');
  const backends = args.length ? args : ['_build/default/web/browser.bc.js', '_build/default/web/browser.bc.wasm.js'].map(file => path.join(root, file));
  const reports = [];
  for (const backend of backends) {
    try { reports.push(await (backends.length > 1 ? runChild(backend) : runBackend(backend, data))); }
    catch (error) { console.error(`${backend}: ${error.stack || error.message}`); process.exitCode = 1; }
  }
  if (reports.some(report => report.failures.length)) process.exitCode = 1;
  if (worker) {
    assert.equal(reports.length, 1, 'Backend worker must produce exactly one report');
    process.send({report: reports[0]});
    return;
  }
  if (reports.length === backends.length && reports.every(report => report.failures.length === 0)) {
    for (const report of reports.slice(1)) {
      assert.deepEqual(report.manifest, reports[0].manifest, 'Backend manifests differ');
      assert.deepEqual(report.coverage, reports[0].coverage, 'Backend execution coverage differs');
      assert.deepEqual(report.results, reports[0].results, 'Backend snapshots/renders/steps/compiler outputs differ');
    }
    console.log(backends.length > 1 ? 'All documented public names reachable; fixture API results agree across backends'
      : 'All documented public names reachable; fixture API checks passed');
  }
}
main().catch(error => { console.error(error.stack || error.message); process.exitCode = 1; });
