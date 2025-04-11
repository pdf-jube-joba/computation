import init, {} from "./pkg/while_minus_lang_web.js";

// ---- wasm module glue code ----

export async function load() {
    await init();
}

// Use this function when you want to load the wasm module
export const ready = new Promise(resolve => {
    document.addEventListener("wasm-ready", resolve);
});

// ---- wasm module glue code end ----