// assets/renderer.js
// Draft: common renderer prototype. Expects an array of JSON blocks.
//
// Supported block kinds: title, className are optional for all block types.
// - text: { kind: "text", text: string }
//   # simple text block
// - table: { kind: "table", columns: [block], rows: [{ className?, cells: [block] }] }
//   # a table with optional header and rows. each cell can be a block.
// - container: { kind: "container", children: [block], orientation: "vertical" | "horizontal", display: "inline" | "block"}
//   # a flat displayed container for grouping blocks. orientation defaults to vertical, display defaults to block.
// Unsupported (for now):
// - graph: { kind: "graph", nodes: [{ id: number, inner: block }], edges: [{ from: number, to: number, inner: block }] }
// - grid: { kind: "grid", cells: [[{ text }]] }
// Blocks may include optional layout hints:
// - className: string (extra CSS class)

export class Renderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
  }

  draw(state) {
    if (!this.stateContainer) return;
    this.stateContainer.replaceChildren();

    if (!Array.isArray(state)) {
      this.stateContainer.appendChild(this.makeText("(invalid render data)"));
      return;
    }

    state.forEach(block => {
      const node = this.renderBlock(block);
      if (node) {
        this.stateContainer.appendChild(node);
      }
    });
  }

  renderBlock(block) {
    if (!block || typeof block !== "object") {
      return this.makeText("(invalid block)");
    }

    const kind = block.kind;
    let node = null;
    switch (kind) {
      case "text":
        node = this.renderText(block);
        break;
      case "table":
        node = this.renderTable(block);
        break;
      case "container":
        node = this.renderContainer(block);
        break;
      default:
        node = this.makeText(`(unknown block kind: ${kind ?? "?"})`);
        break;
    }

    if (!node) return null;
    node.classList.add("wm-block");
    if (typeof block.className === "string" && block.className) {
      block.className.split(/\s+/).forEach(cls => node.classList.add(cls));
    }
    return node;
  }

  renderText(block) {
    const wrapper = this.makeSection(block.title);
    const p = document.createElement("div");
    p.className = "wm-text";
    p.textContent = block.text ?? "";
    wrapper.appendChild(p);
    return wrapper;
  }

  renderTable(block) {
    const wrapper = this.makeSection(block.title);
    const table = document.createElement("table");
    table.className = "wm-table";

    const columns = Array.isArray(block.columns) ? block.columns : [];
    if (columns.length) {
      const thead = table.createTHead();
      const headRow = thead.insertRow();
      columns.forEach(col => {
        const cell = document.createElement("th");
        const content = this.renderBlock(col);
        if (content) {
          cell.appendChild(content);
        }
        headRow.appendChild(cell);
      });
    }

    const rows = Array.isArray(block.rows) ? block.rows : [];
    const tbody = table.createTBody();
    rows.forEach(row => {
      const tr = tbody.insertRow();
      if (row?.className) {
        tr.className = row.className;
      }
      const cells = Array.isArray(row?.cells) ? row.cells : [];
      cells.forEach(cell => {
        const td = document.createElement("td");
        const content = this.renderBlock(cell);
        if (content) {
          td.appendChild(content);
        }
        tr.appendChild(td);
      });
    });

    wrapper.appendChild(table);
    return wrapper;
  }

  renderContainer(block) {
    const wrapper = this.makeSection(block.title);
    const container = document.createElement("div");
    container.className = "wm-container";
    const display = block.display === "inline" ? "inline-flex" : "flex";
    const direction = block.orientation === "horizontal" ? "row" : "column";
    container.style.display = display;
    container.style.flexDirection = direction;
    const children = Array.isArray(block.children) ? block.children : [];
    children.forEach(child => {
      const node = this.renderBlock(child);
      if (node) {
        container.appendChild(node);
      }
    });
    wrapper.appendChild(container);
    return wrapper;
  }

  makeSection(title) {
    const section = document.createElement("section");
    section.className = "wm-section";
    if (title) {
      const heading = document.createElement("h4");
      heading.className = "wm-section-title";
      heading.textContent = title;
      section.appendChild(heading);
    }
    return section;
  }

  makeText(text) {
    const div = document.createElement("div");
    div.className = "wm-text";
    div.textContent = text ?? "";
    return div;
  }
}
