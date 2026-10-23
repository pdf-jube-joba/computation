// Match web/ui/script.js's model API without requiring a Wasm Component.
// Each factory call creates its own model session.
export function createAdapter(api, name) {
  const session = api.model(name);
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
      return JSON.stringify({kind: result.kind, [field]: result.output});
    },
    snapshot() { return JSON.stringify(unwrap(session.snapshot())); },
    restore(snapshot) { unwrap(session.restore(snapshot)); },
    render(snapshot) { return JSON.stringify(unwrap(session.renderSnapshot(snapshot))); },
  };
}

export function createCompilerAdapter(api, name) {
  const compiler = api.compiler(name);
  const call = method => input => {
    const result = JSON.parse(compiler[method](input));
    if ('error' in result) throw new Error(result.error);
    return result.ok;
  };
  return Object.fromEntries(['compileCode', 'encodeAinput', 'encodeRinput',
    'decodeRoutput', 'decodeFoutput'].map(method => [method, call(method)]));
}
