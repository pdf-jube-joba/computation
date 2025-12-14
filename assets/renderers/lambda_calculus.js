// assets/renderers/lambda_calculus.js
// SnapshotRenderer for lambda calculus marked terms with simple redex highlighting

const REDEX_COLORS = ["#ffe7c2", "#ffd4db", "#e4f2ff", "#e9ffd0", "#f5d0ff"];

export class SnapshotRenderer {
  constructor(stateContainer) {
    this.stateContainer = stateContainer;
    this.stateContainer.replaceChildren();

    this.infoLabel = document.createElement("div");
    this.infoLabel.className = "lambda-help";
    this.infoLabel.textContent =
      "Redex indices are assigned depth-first (input = index you want to reduce).";

    this.termContainer = document.createElement("div");
    this.termContainer.className = "lambda-term";

    this.stateContainer.append(this.infoLabel, this.termContainer);
  }

  draw(state) {
    if (!this.termContainer) return;
    this.termContainer.replaceChildren();
    if (!state || typeof state !== "object") {
      this.termContainer.textContent = "(no term)";
      if (this.infoLabel) {
        this.infoLabel.textContent = "Redex: 0 (normal form)";
      }
      return;
    }

    const counter = { nextIndex: 0 };
    const node = this.renderTerm(state, counter);
    if (node) {
      this.termContainer.appendChild(node);
    } else {
      this.termContainer.textContent = "(term unavailable)";
    }
    if (this.infoLabel) {
      if (counter.nextIndex === 0) {
        this.infoLabel.textContent = "Redex: 0 (normal form)";
      } else if (counter.nextIndex === 1) {
        this.infoLabel.textContent =
          "Redex: 1 (input 0 to reduce the highlighted application)";
      } else {
        this.infoLabel.textContent = `Redex: ${counter.nextIndex} (input 0..${
          counter.nextIndex - 1
        })`;
      }
    }
  }

  renderTerm(node, counter) {
    const variant = this.extractVariant(node);
    if (!variant) {
      return this.textSpan(String(node ?? ""));
    }
    const { tag, value } = variant;
    switch (tag) {
      case "Var":
        return this.renderVar(value);
      case "Abs":
        return this.renderAbstraction(value, counter);
      case "App":
        return this.renderApplication(value, counter);
      case "Red":
        return this.renderRedex(value, counter);
      default:
        return this.textSpan(`[${tag}]`);
    }
  }

  renderAbstraction(value, counter) {
    const [varName, body] = Array.isArray(value) ? value : [value];
    const span = document.createElement("span");
    span.className = "lambda-abs";
    span.append("(");
    span.append(this.textSpan("\\"));
    span.append(this.renderVar(varName));
    span.append(this.textSpan(". "));
    span.append(this.renderTerm(body, counter));
    span.append(")");
    return span;
  }

  renderApplication(value, counter) {
    const [lhs, rhs] = Array.isArray(value) ? value : [value];
    const span = document.createElement("span");
    span.className = "lambda-app";
    span.append("(");
    span.append(this.renderTerm(lhs, counter));
    span.append(this.textSpan(" "));
    span.append(this.renderTerm(rhs, counter));
    span.append(")");
    return span;
  }

  renderRedex(value, counter) {
    const [varName, body, arg] = Array.isArray(value) ? value : [value];
    const idx = counter.nextIndex++;
    const wrapper = document.createElement("span");
    wrapper.className = "lambda-redex";
    wrapper.style.backgroundColor = REDEX_COLORS[idx % REDEX_COLORS.length];
    wrapper.style.border = "1px solid rgba(0, 0, 0, 0.2)";
    wrapper.style.borderRadius = "4px";
    wrapper.style.padding = "0 4px";
    wrapper.style.margin = "0 2px";

    const badge = document.createElement("span");
    badge.className = "lambda-redex-badge";
    badge.style.fontSize = "0.8em";
    badge.style.fontWeight = "bold";
    badge.style.marginRight = "4px";
    badge.textContent = `#${idx}`;

    const app = document.createElement("span");
    app.className = "lambda-redex-app";
    app.append("(");
    app.append(this.renderAbstraction([varName, body], counter));
    app.append(this.textSpan(" "));
    app.append(this.renderTerm(arg, counter));
    app.append(")");

    wrapper.append(badge, app);
    return wrapper;
  }

  extractVariant(node) {
    if (!node || typeof node !== "object") return null;
    const keys = Object.keys(node);
    if (keys.length !== 1) return null;
    const tag = keys[0];
    return { tag, value: node[tag] };
  }

  textSpan(text, className) {
    const span = document.createElement("span");
    if (className) span.classList.add(className);
    span.textContent = text;
    return span;
  }

  renderVar(value) {
    const { text, title } = this.formatVar(value);
    const span = this.textSpan(text, "lambda-var");
    if (title) span.title = title;
    return span;
  }

  formatVar(value) {
    if (value && typeof value === "object") {
      const name = value.name ?? "?";
      const ptr = value.ptr;
      const text = String(name);
      const title = ptr !== undefined ? `unique id based on pointer: ${ptr}` : undefined;
      return { text, title };
    }
    return { text: String(value ?? "?"), title: undefined };
  }
}
