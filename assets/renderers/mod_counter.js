export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
    stateContainer.replaceChildren();
  }

  draw(state) {
    if (!state) return;
    const remainder = formatNumber(state.remainder, "0");
    const count = formatNumber(state.count, "0");
    this.stateContainer.textContent = `count = ${count}, remainder = ${remainder}`;
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
