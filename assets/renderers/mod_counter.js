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
