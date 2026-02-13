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
    target.textContent = `count = ${formatNumber(raw, "0")}`;
  }
}

function formatNumber(raw, fallback) {
  const big = toBigInt(raw);
  return big == null ? fallback : big.toString(10);
}

function toBigInt(raw) {
  if (typeof raw === "bigint") return raw;
  if (typeof raw === "number" && Number.isFinite(raw)) {
    return BigInt(Math.trunc(raw));
  }
  if (typeof raw === "string" && raw.trim() !== "") {
    try {
      return BigInt(raw);
    } catch (_) {
      return null;
    }
  }
  return null;
}
