// assets/renderers/turing_machine.js
// Renderer for Turing machine state/output
export class Renderer {
  constructor(vm, stateContainer, outputContainer) {
    this.vm = vm;
    this.stateContainer = stateContainer;
    this.outputContainer = outputContainer;

    this.stateContainer.replaceChildren();
    this.outputContainer.replaceChildren();

    this.statusDiv = document.createElement("div");
    this.tapeDiv = document.createElement("div");
    this.codeTable = document.createElement("table");

    const thead = this.codeTable.createTHead();
    const headRow = thead.insertRow();
    ["key_sign", "key_state", "next_sign", "next_state", "direction"].forEach(text => {
      headRow.insertCell().innerText = text;
    });

    this.stateContainer.append(this.statusDiv, this.tapeDiv, this.codeTable);

    this.outputLabel = document.createElement("div");
    this.outputContainer.append(this.outputLabel);
  }

  drawState(state) {
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

  drawOutput(output) {
    if (!output) {
      this.outputLabel.textContent = "";
      return;
    }
    this.outputLabel.textContent = `output: ${JSON.stringify(output)}`;
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
  const toCells = arr => (Array.isArray(arr) ? arr : []);
  const norm = c => (c && c.trim() !== "" ? c : " ");
  const left = toCells(tape.left).slice().reverse().map(norm);
  const right = toCells(tape.right).map(norm);
  const renderCells = cells => cells.map(c => `[${c}]`);
  const parts = [...renderCells(left), `{${tape.head ?? " "}}`, ...renderCells(right)];
  return parts.join(" ");
}
