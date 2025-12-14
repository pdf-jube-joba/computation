// assets/renderers/mod_counter.js
// Renderer for mod_counter machine
export class Renderer {
  constructor(vm, stateContainer, outputContainer) {
    this.vm = vm;
    this.stateContainer = stateContainer;
    this.outputContainer = outputContainer;
    stateContainer.replaceChildren();
    outputContainer.replaceChildren();
  }

  drawState(state) {
    if (!state) return;
    const remainder = typeof state.remainder === "number" ? state.remainder : Number(state.remainder) || 0;
    const count = typeof state.count === "number" ? state.count : Number(state.count) || 0;
    this.stateContainer.textContent = `count = ${count}, remainder = ${remainder}`;
  }

  drawOutput(output) {
    if (output === undefined) {
      this.outputContainer.textContent = "";
      return;
    }
    const wrapped = !!output.wrapped;
    const count = typeof output.count === "number" ? output.count : Number(output.count) || 0;
    this.outputContainer.textContent = wrapped ? `wrapped at count ${count}` : "";
  }
}
