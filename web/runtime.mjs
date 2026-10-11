// One OCaml runtime per backend, many independent model sessions.
import {createAdapter, createCompilerAdapter} from './model_adapter.mjs';

const runtimes = new Map();
function loadRuntime(backend) {
  if (!runtimes.has(backend)) {
    const promise = new Promise((resolve, reject) => {
      const exportName = backend === 'js' ? 'computationOcamlJs' : 'computationOcamlWasm';
      if (globalThis[exportName]) { resolve(globalThis[exportName]); return; }
      const script = document.createElement('script');
      const loader = backend === 'js' ? 'browser.bc.js' : 'browser.bc.wasm.js';
      script.src = new URL(loader, import.meta.url).href;
      let timer;
      const timeout = setTimeout(() => finish(new Error(`OCaml ${backend} initialization timed out`)), 30000);
      function finish(error, api) {
        clearTimeout(timeout); clearInterval(timer);
        if (error) { script.remove(); reject(error); } else resolve(api);
      }
      script.onerror = () => finish(new Error(`Failed to load ${script.src}`));
      timer = setInterval(() => {
        if (globalThis[exportName]) finish(null, globalThis[exportName]);
      }, 20);
      document.head.appendChild(script);
    });
    runtimes.set(backend, promise);
    // Allow retry when loading the script failed.
    promise.catch(() => runtimes.delete(backend));
  }
  return runtimes.get(backend);
}

export async function createModel(name, backend = 'wasm') {
  return createAdapter(await loadRuntime(backend), name);
}

export async function createCompiler(name, backend = 'wasm') {
  return createCompilerAdapter(await loadRuntime(backend), name);
}
