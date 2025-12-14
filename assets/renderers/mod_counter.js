export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
    stateContainer.replaceChildren();
  }

  draw(state) {
    if (!state) return;
    const remainder = typeof state.remainder === "number" ? state.remainder : Number(state.remainder) || 0;
    const count = typeof state.count === "number" ? state.count : Number(state.count) || 0;
    this.stateContainer.textContent = `count = ${count}, remainder = ${remainder}`;
  }
}

export class OutputRenderer {
  constructor(outputContainer) {
    this.outputContainer = outputContainer;
    outputContainer.replaceChildren();
  }

  draw(output) {
    if (output === undefined) {
      this.outputContainer.textContent = "";
      return;
    }
    const wrapped = !!output.wrapped;
    const count = typeof output.count === "number" ? output.count : Number(output.count) || 0;
    this.outputContainer.textContent = wrapped ? `wrapped at count ${count}` : "";
  }
}
