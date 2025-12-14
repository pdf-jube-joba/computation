// assets/renderers/turing_machine.js
// SnapshotRenderer for Turing machine state/output
export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;

    this.stateContainer.replaceChildren();

    this.statusDiv = document.createElement("div");
    this.tapeDiv = document.createElement("div");
    this.codeTable = document.createElement("table");

    const thead = this.codeTable.createTHead();
    const headRow = thead.insertRow();
    ["key_sign", "key_state", "next_sign", "next_state", "direction"].forEach(text => {
      headRow.insertCell().innerText = text;
    });

    this.stateContainer.append(this.statusDiv, this.tapeDiv, this.codeTable);
  }

  draw(state) {
    if (!state) {
      this.statusDiv.textContent = "(no state)";
      this.tapeDiv.textContent = "";
      this.clearTableBody();
      return;
    }

    const terminateFlag = state.output && state.output.terminate === true;
    const now = typeof state.now === "number" ? state.now : null;
    const statusParts = [`state: ${state.state ?? "-"}`];
    if (now !== null) statusParts.push(`now: ${now}`);
    if (terminateFlag) statusParts.push("(terminated)");
    this.statusDiv.textContent = statusParts.join(" | ");

    this.tapeDiv.textContent = formatTape(state.tape);
    this.fillCodeTable(state.code || [], now);
  }

  clearTableBody() {
    while (this.codeTable.tBodies.length) {
      this.codeTable.removeChild(this.codeTable.tBodies[0]);
    }
  }

  fillCodeTable(entries, now) {
    this.clearTableBody();
    const tbody = this.codeTable.createTBody();
    entries.forEach((entry, idx) => {
      const row = tbody.insertRow();
      if (idx === now) {
        row.style.backgroundColor = "#eef";
      }
      [
        entry.key_sign,
        entry.key_state,
        entry.next_sign,
        entry.next_state,
        entry.direction,
      ].forEach(text => {
        row.insertCell().innerText = text ?? "";
      });
    });
  }
}

function formatTape(tape) {
  if (!tape) return "(no tape)";
  const WINDOW = 7;
  const mid = Math.floor(WINDOW / 2);
  const norm = c => {
    const s = (c ?? "").toString();
    return s.trim() === "" ? " " : s;
  };
  const left = Array.isArray(tape.left) ? tape.left : [];
  const right = Array.isArray(tape.right) ? tape.right : [];
  const cells = Array.from({ length: WINDOW }, () => " ");

  // fill left side (closest to head to the left)
  for (let i = 1; i <= mid; i++) {
    const val = left[left.length - i];
    if (val !== undefined) {
      cells[mid - i] = norm(val);
    }
  }

  // fill right side (closest to head to the right)
  for (let i = 1; i < WINDOW - mid; i++) {
    const val = right[right.length - i];
    if (val !== undefined) {
      cells[mid + i] = norm(val);
    }
  }

  const head = norm(tape.head);
  return cells
    .map((c, idx) => (idx === mid ? `{${head}}` : `[${c}]`))
    .join(" ");
}
