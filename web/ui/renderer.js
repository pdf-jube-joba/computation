// web/ui/renderer.js
// Draft: common renderer prototype. Expects an array of JSON blocks.
//
// Supported block kinds: title, className are optional for all block types.
// - text: { kind: "text", text: string }
//   # simple text block
// - table: { kind: "table", columns: [block], rows: [{ className?, cells: [block] }] }
//   # a table with optional header and rows. each cell can be a block.
// - container: { kind: "container", children: [block], orientation: "vertical" | "horizontal", display: "inline" | "block"}
//   # a flat displayed container for grouping blocks. orientation defaults to vertical, display defaults to block.
// - graph: { kind: "graph", nodes: [{ id, label, ports: [string] }],
//   edges: [{ source: {node, port}, target: {node, port}, highlighted: boolean }] }
// Unsupported (for now):
// - grid: { kind: "grid", cells: [[{ text }]] }
// Blocks may include optional layout hints:
// - className: string (extra CSS class)

export class Renderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
  }

  draw(state) {
    console.log("Rendering state:", state);

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
      case "graph":
        node = this.renderGraph(block);
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
        if (block.display === "block") {
          const item = document.createElement("div");
          item.className = "wm-container-item";
          item.style.border = "1px solid #000";
          item.style.padding = "8px";
          item.style.margin = "8px 0";
          item.appendChild(node);
          container.appendChild(item);
        } else {
          container.appendChild(node);
        }
      }
    });
    wrapper.appendChild(container);
    return wrapper;
  }

  renderGraph(block) {
    const wrapper = this.makeSection(block.title);
    wrapper.classList.add("wm-graph");
    const nodes = Array.isArray(block.nodes) ? block.nodes : [];
    const edges = Array.isArray(block.edges) ? block.edges : [];
    if (nodes.length === 0) {
      wrapper.appendChild(this.makeText("∅"));
      return wrapper;
    }
    const svgElement = (name, attributes = {}, text = null) => {
      const element = document.createElementNS("http://www.w3.org/2000/svg", name);
      for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, String(value));
      if (text !== null) element.textContent = text;
      return element;
    };
    const columns = Math.ceil(Math.sqrt(nodes.length));
    const width = columns * 190;
    const height = Math.ceil(nodes.length / columns) * 180;
    const svg = svgElement("svg", {viewBox: `0 0 ${width} ${height}`, width, height,
      role: "img", "aria-label": block.title || "Port graph"});
    svg.appendChild(svgElement("title", {}, block.title || "Port graph"));
    const positions = new Map();
    nodes.forEach((node, index) => {
      const x = 95 + (index % columns) * 190;
      const y = 95 + Math.floor(index / columns) * 180;
      const ports = (Array.isArray(node.ports) ? node.ports : []).map((label, i, list) => {
        const angle = -Math.PI / 2 + i * 2 * Math.PI / list.length;
        const dx = Math.cos(angle), dy = Math.sin(angle);
        return {x: x + 32 * dx, y: y + 32 * dy, dx, dy, label};
      });
      positions.set(node.id, {x, y, ports});
    });
    const endpoint = end => positions.get(end?.node)?.ports[end?.port];
    const edgeLayer = svgElement("g", {class: "wm-graph-edges"});
    edges.forEach(edge => {
      const a = endpoint(edge.source), b = endpoint(edge.target);
      if (!a || !b) return;
      // Controls follow each port's outward direction, separating parallel
      // wires and keeping connections to the same node visible.
      const bend = edge.source.node === edge.target.node ? 95 : 60;
      const path = svgElement("path", {
        d: `M ${a.x} ${a.y} C ${a.x + bend * a.dx} ${a.y + bend * a.dy}, ${b.x + bend * b.dx} ${b.y + bend * b.dy}, ${b.x} ${b.y}`,
        class: edge.highlighted ? "wm-graph-edge is-active" : "wm-graph-edge",
      });
      path.appendChild(svgElement("title", {},
        `${edge.source.node}.${edge.source.port} ↔ ${edge.target.node}.${edge.target.port}`));
      edgeLayer.appendChild(path);
    });
    svg.appendChild(edgeLayer);
    nodes.forEach(node => {
      const {x, y, ports} = positions.get(node.id);
      const group = svgElement("g", {class: "wm-graph-node"});
      group.appendChild(svgElement("circle", {cx: x, cy: y, r: 28}));
      group.appendChild(svgElement("text", {x, y: y + 5, "text-anchor": "middle"}, node.label));
      ports.forEach((port, i) => {
        const circle = svgElement("circle", {cx: port.x, cy: port.y, r: i === 0 ? 5 : 3.5,
          class: "wm-graph-port"});
        circle.appendChild(svgElement("title", {}, `${node.label}: ${port.label}`));
        group.appendChild(circle);
        group.appendChild(svgElement("text", {x: port.x + 13 * port.dx,
          y: port.y + 13 * port.dy + 4, "text-anchor": "middle", class: "wm-graph-port-label"}, port.label));
      });
      svg.appendChild(group);
    });
    wrapper.appendChild(svg);
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
