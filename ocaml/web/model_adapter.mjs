// Match assets/script.js's model API without requiring a Wasm Component.
// Each module import (including ?instance=...) creates its own model session.
export function createAdapter(api, name) {
  const factory = api[name];
  if (typeof factory !== 'function') throw new Error(`Unknown OCaml model: ${name}`);
  const session = factory();
  function unwrap(text) {
    const result = JSON.parse(text);
    if ('error' in result) throw new Error(result.error);
    return result.ok;
  }
  return {
    make(code, ainput) { unwrap(session.make(code, ainput)); },
    step(input) {
      const result = unwrap(session.step(input));
      const field = result.kind === 'continue' ? 'routput' : 'foutput';
      const output = result.output === null ? '' : result.output;
      return JSON.stringify({kind: result.kind, [field]: output});
    },
    snapshot() { return JSON.stringify(unwrap(session.snapshot())); },
    restore(snapshot) { unwrap(session.restore(snapshot)); },
    render(snapshot) { return JSON.stringify(unwrap(session.renderSnapshot(snapshot))); },
  };
}
