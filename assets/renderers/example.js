// SnapshotRenderer only
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
