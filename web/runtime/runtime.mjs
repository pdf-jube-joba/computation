// One shared OCaml Wasm runtime, many independent model sessions.
import {createAdapter, createCompilerAdapter} from './model_adapter.mjs';

let runtime;
function loadRuntime() {
  if (!runtime) {
    runtime = new Promise((resolve, reject) => {
      const exportName = 'computationOcamlWasm';
      if (globalThis[exportName]) { resolve(globalThis[exportName]); return; }
      const script = document.createElement('script');
      script.src = new URL('browser.bc.wasm.js', import.meta.url).href;
      let timer;
      const timeout = setTimeout(() => finish(new Error('OCaml Wasm initialization timed out')), 30000);
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
    // Allow retry when loading the script failed.
    runtime.catch(() => { runtime = undefined; });
  }
  return runtime;
}

export async function createModel(name) {
  return createAdapter(await loadRuntime(), name);
}

export async function createCompiler(name) {
  return createCompilerAdapter(await loadRuntime(), name);
}
