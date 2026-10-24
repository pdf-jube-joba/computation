// Exercise SVG construction without a browser or third-party DOM package.
const assert = require('node:assert/strict');
const {readFileSync} = require('node:fs');
const path = require('node:path');
const {test} = require('node:test');

class Element {
  constructor(name, namespace = null) {
    this.name = name;
    this.namespace = namespace;
    this.children = [];
    this.attributes = {};
    this.textContent = '';
    this.classList = {add: (...names) => {
      this.attributes.class = [this.attributes.class || '', ...names].join(' ').trim();
    }};
  }
  setAttribute(name, value) { this.attributes[name] = String(value); }
  appendChild(child) { this.children.push(child); return child; }
}

global.document = {
  createElement: name => new Element(name),
  createElementNS: (ns, name) => new Element(name, ns),
};
const source = readFileSync(path.join(__dirname, '../web/ui/renderer.js'), 'utf8');
const renderer = import('data:text/javascript;base64,' + Buffer.from(source).toString('base64'));
const descendants = element => [element, ...element.children.flatMap(descendants)];

test('port graph keeps all wires, labels, and the active pair in SVG', async () => {
  const {Renderer} = await renderer;
  const result = new Renderer(null).renderBlock({
    kind: 'graph', title: 'net',
    nodes: [
      {id: 'a', label: 'γ a', ports: ['0', '1', '2']},
      {id: 'b', label: 'δ b', ports: ['0', '1', '2']},
      {id: 'out', label: '<script>literal</script>', ports: ['']},
    ],
    edges: [
      {source: {node: 'a', port: 0}, target: {node: 'b', port: 0}, highlighted: true},
      {source: {node: 'a', port: 1}, target: {node: 'a', port: 2}, highlighted: false},
      {source: {node: 'b', port: 1}, target: {node: 'b', port: 2}, highlighted: false},
    ],
  });
  const all = descendants(result);
  const svg = all.find(e => e.name === 'svg');
  assert.equal(svg.namespace, 'http://www.w3.org/2000/svg');
  assert.equal(svg.attributes.role, 'img');
  const paths = all.filter(e => e.name === 'path');
  assert.equal(paths.length, 3);
  assert.equal(paths.filter(e => e.attributes.class.includes('is-active')).length, 1);
  assert.equal(new Set(paths.map(e => e.attributes.d)).size, 3);
  assert.ok(paths.every(e => !e.attributes.d.includes('NaN')));
  assert.ok(paths[0].children.some(e => e.textContent === 'a.0 ↔ b.0'));
  assert.ok(paths[1].children.some(e => e.textContent === 'a.1 ↔ a.2'));
  assert.equal(all.filter(e => e.name === 'circle').length, 10);
  assert.ok(all.some(e => e.name === 'text' && e.textContent === '<script>literal</script>'));
  assert.ok(!all.some(e => e.name === 'script'));
});

test('empty graph renders its empty state', async () => {
  const {Renderer} = await renderer;
  const result = new Renderer(null).renderBlock({kind: 'graph', nodes: [], edges: []});
  assert.ok(descendants(result).some(e => e.textContent === '∅'));
  assert.ok(!descendants(result).some(e => e.name === 'svg'));
});
