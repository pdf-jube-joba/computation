// Optional integration check against a running _wfs client. Execute generated
// browser loaders in a VM without Node globals, using the real HTTP endpoints.
const assert = require('node:assert/strict');
const vm = require('node:vm');
const origin = process.argv[2] || 'http://localhost:3000';
const loadedScripts = [];
const context = vm.createContext({console, URL, fetch, TextDecoder, TextEncoder,
  performance, crypto, setTimeout, clearTimeout, setInterval, clearInterval});
context.window = context;
context.self = context;
context.location = {href: `${origin}/md/md_preview.html?path=ocaml/demo.md`};
context.document = {
  compatMode: 'CSS1Compat',
  currentScript: null,
  createElement() { return {remove() {}}; },
  head: {
    appendChild(script) {
      loadedScripts.push(script.src);
      void (async () => {
        const source = await get(script.src);
        context.document.currentScript = script;
        try { vm.runInContext(source, context, {filename: script.src}); }
        finally { context.document.currentScript = null; }
      })().catch(error => { console.error(error); script.onerror(); });
    },
  },
};

async function get(url) {
  const response = await fetch(url);
  assert.equal(response.status, 200, `${url}: ${response.status}`);
  return response.text();
}
const modules = new Map();
function load(url) {
  if (!modules.has(url)) {
    const promise = (async () => {
      const source = await get(url);
      const module = new vm.SourceTextModule(source, {
        context, identifier: url,
        initializeImportMeta(meta) { meta.url = url; },
        async importModuleDynamically(specifier, referring) {
          const module = await load(new URL(specifier, referring.identifier).href);
          if (module.status === 'linked') await module.evaluate();
          return module;
        },
      });
      await module.link((specifier, referring) => load(new URL(specifier, referring.identifier).href));
      return module;
    })();
    modules.set(url, promise);
  }
  return modules.get(url);
}
async function importModel(name, instance) {
  const module = await load(`${origin}/ocaml_bundle/${name}.js?instance=${instance}`);
  await module.evaluate();
  return module.namespace;
}

(async () => {
  const preview = await get(`${origin}/md/md_preview.html`);
  assert.ok(preview.includes('/assets/header.js'));
  const markdown = await get(`${origin}/ocaml/demo.md`);
  const viewer = await load(`${origin}/md/markdown_viewer.js`);
  await viewer.evaluate();
  const html = await viewer.namespace.renderMarkdownToHtml({text: markdown, basePath: 'ocaml/demo.md', macros: {}});
  for (const name of ['ocaml_example_counter', 'ocaml_example_counter_js', 'ocaml_lambda_calculus']) {
    assert.ok(html.includes(`data-model="${name}"`), `Missing ${name} in rendered Markdown`);
  }
  assert.ok(html.includes('data-bundle-base-url="/ocaml_bundle/"'));
  assert.ok(html.includes('data-default-code'));
  for (const suffix of ['', '_js']) {
    const a = await importModel(`ocaml_example_counter${suffix}`, `a${suffix}`);
    const b = await importModel(`ocaml_example_counter${suffix}`, `b${suffix}`);
    a.make('8', ''); b.make('2', '');
    const snapshot = a.snapshot();
    assert.equal(JSON.parse(a.render(snapshot))[0].rows[0].cells[1].text, '8');
    assert.equal(JSON.parse(a.step('inc')).routput, 'inc');
    assert.equal(JSON.parse(b.snapshot()).count, 2);
    assert.equal(JSON.parse(a.step('inc')).foutput, 'End');
    a.restore(snapshot);
    assert.equal(a.snapshot(), snapshot);
    const lambda = await importModel(`ocaml_lambda_calculus${suffix}`, `lambda${suffix}`);
    lambda.make('(\\x. x) (\\y. y)', '');
    assert.equal(JSON.parse(lambda.step('0')).foutput, '\\y. y');
  }
  assert.equal(loadedScripts.length, 2, 'Each backend must load only once');
  console.log('_wfs HTTP, Markdown, browser loaders, and model API integration passed');
})().catch(error => { console.error(error); process.exitCode = 1; });
