// assets/renderers/example.js
// Renderer class with drawState/drawOutput
export class Renderer {
  constructor(vm, stateContainer, outputContainer) {
    this.vm = vm;
    this.stateContainer = stateContainer;
    this.outputContainer = outputContainer;

    this.stateContainer.replaceChildren();
    this.outputContainer.replaceChildren();
    const stateSpan = document.createElement("span");
    stateSpan.classList.add("count-view");
    this.stateContainer.appendChild(stateSpan);

    const outputSpan = document.createElement("span");
    outputSpan.classList.add("output-view");
    this.outputContainer.appendChild(outputSpan);
  }

  drawState(state) {
    const target = this.stateContainer.querySelector(".count-view");
    if (!target) return;
    const raw = state instanceof Map ? state.get("count") : state && state.count;
    const count = typeof raw === "number" ? raw : Number(raw) || 0;
    target.textContent = `count = ${count}`;
  }

  drawOutput(output) {
    // if output === undefined, clear the output view
    const target = this.outputContainer.querySelector(".output-view");
    if (!target) return;
    console.log("Drawing output:", output);

    if (output === undefined) {
      target.textContent = "";
      return;
    }

    target.textContent = output ? `output: ${output}` : "";
  }
}
