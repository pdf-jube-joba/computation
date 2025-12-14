// SnapshotRenderer + OutputRenderer split
export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
    this.stateContainer.replaceChildren();
    const stateSpan = document.createElement("span");
    stateSpan.classList.add("count-view");
    this.stateContainer.appendChild(stateSpan);
  }

  draw(state) {
    const target = this.stateContainer.querySelector(".count-view");
    if (!target) return;
    const raw = state instanceof Map ? state.get("count") : state && state.count;
    const count = typeof raw === "number" ? raw : Number(raw) || 0;
    target.textContent = `count = ${count}`;
  }
}

export class OutputRenderer {
  constructor(outputContainer) {
    this.outputContainer = outputContainer;
    this.outputContainer.replaceChildren();
    const outputSpan = document.createElement("span");
    outputSpan.classList.add("output-view");
    this.outputContainer.appendChild(outputSpan);
  }

  draw(output) {
    const target = this.outputContainer.querySelector(".output-view");
    if (!target) return;
    if (output === undefined) {
      target.textContent = "";
      return;
    }
    target.textContent = output ? `output: ${output}` : "";
  }
}
