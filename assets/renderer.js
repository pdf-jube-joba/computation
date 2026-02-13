// assets/renderer.js
// Draft: common renderer prototype. Expects an array of JSON blocks.
//
// Supported block kinds (initial sketch):
// - text: { kind: "text", text: "..." }
// - kv: { kind: "kv", title?: "...", items: [{ key, value }] }
// - table: { kind: "table", title?: "...", columns: [""], rows: [[...]] }
// - code: { kind: "code", title?: "...", lines: [""], highlightIndex?: number }
// - list: { kind: "list", title?: "...", items: [""|object] }
// - tree: { kind: "tree", title?: "...", root: { label, children?: [...] } }
// Unsupported block kinds
// - graph: { kind: "graph", title?: "...", nodes: [{ id, label }], edges: [{ from, to }] }
// - stack: { kind: "stack", title?: "...", frames: [{ text }] }
// - grid: { kind: "grid", title?: "...", cells: [[{ text }]] }
// Blocks may include optional layout hints:
// - order: number (for CSS ordering)
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
      case "kv":
        node = this.renderKV(block);
        break;
      case "table":
        node = this.renderTable(block);
        break;
      case "code":
        node = this.renderCode(block);
        break;
      case "list":
        node = this.renderList(block);
        break;
      case "tree":
        node = this.renderTree(block);
        break;
      default:
        node = this.makeText(`(unknown block kind: ${kind ?? "?"})`);
        break;
    }

    if (!node) return null;
    node.classList.add("wm-block");
    if (typeof block.order === "number") {
      node.dataset.order = String(block.order);
    }
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

  renderKV(block) {
    const wrapper = this.makeSection(block.title);
    const list = document.createElement("dl");
    list.className = "wm-kv";
    const items = Array.isArray(block.items) ? block.items : [];
    items.forEach(item => {
      const dt = document.createElement("dt");
      dt.textContent = item?.key ?? "";
      const dd = document.createElement("dd");
      dd.textContent = item?.value ?? "";
      list.append(dt, dd);
    });
    wrapper.appendChild(list);
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
        headRow.insertCell().textContent = col ?? "";
      });
    }

    const rows = Array.isArray(block.rows) ? block.rows : [];
    const tbody = table.createTBody();
    rows.forEach(row => {
      const tr = tbody.insertRow();
      const cells = Array.isArray(row) ? row : [];
      cells.forEach(cell => {
        tr.insertCell().textContent = cell ?? "";
      });
    });

    wrapper.appendChild(table);
    return wrapper;
  }

  renderCode(block) {
    const wrapper = this.makeSection(block.title);
    const list = document.createElement("ol");
    list.className = "wm-code";
    const lines = Array.isArray(block.lines) ? block.lines : [];
    const highlight = typeof block.highlightIndex === "number" ? block.highlightIndex : null;
    lines.forEach((line, idx) => {
      const li = document.createElement("li");
      li.textContent = line ?? "";
      if (highlight === idx) {
        li.classList.add("wm-code-current");
      }
      list.appendChild(li);
    });
    wrapper.appendChild(list);
    return wrapper;
  }

  renderList(block) {
    const wrapper = this.makeSection(block.title);
    const list = document.createElement("ul");
    list.className = "wm-list";
    const items = Array.isArray(block.items) ? block.items : [];
    items.forEach(item => {
      const li = document.createElement("li");
      if (item && typeof item === "object") {
        li.textContent = item.text ?? JSON.stringify(item);
      } else {
        li.textContent = item ?? "";
      }
      list.appendChild(li);
    });
    wrapper.appendChild(list);
    return wrapper;
  }

  renderTree(block) {
    const wrapper = this.makeSection(block.title);
    const container = document.createElement("div");
    container.className = "wm-tree";
    const root = block.root;
    if (root) {
      container.appendChild(this.renderTreeNode(root));
    } else {
      container.textContent = "(empty tree)";
    }
    wrapper.appendChild(container);
    return wrapper;
  }

  renderTreeNode(node) {
    const item = document.createElement("div");
    item.className = "wm-tree-node";

    const label = document.createElement("div");
    label.className = "wm-tree-label";
    label.textContent = node?.label ?? "";
    item.appendChild(label);

    const children = Array.isArray(node?.children) ? node.children : [];
    if (children.length) {
      const list = document.createElement("div");
      list.className = "wm-tree-children";
      children.forEach(child => {
        list.appendChild(this.renderTreeNode(child));
      });
      item.appendChild(list);
    }

    return item;
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
