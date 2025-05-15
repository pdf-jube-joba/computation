import init, { new_logic_circuit, set_logic_circuit, get_logic_circuit, step_logic_circuit, PinWeb } from "./pkg/logic_circuit_web.js";

// ---- wasm module glue code ----

export async function load() {
  await init();
}

// Use this function when you want to load the wasm module
export const ready = new Promise(resolve => {
  document.addEventListener("wasm-ready", resolve);
});

// ---- wasm module glue code end ----

// ---- view models class ----
export class LogicCircuitViewModel {
  machineId = undefined;

  constructor(codeResource, controls, viewId, default_placement) {
    // control: UserControls
    this.codeResource = codeResource;
    this.view = new LogicCircuitView(viewId, default_placement);
    this.controls = controls;
    this.controls.setOnLoad(() => {
      this.start();
    });
    this.controls.setOnStep(() => {
      this.step();
    });
  }

  start() {
    const text = this.codeResource.getText();
    console.log("load code", text);
    if (!text) {
      this.controls.handleError("Please write code");
      return;
    };
    try {
      if (this.machineId == undefined) {
        this.machineId = new_logic_circuit(text);
      } else {
        set_logic_circuit(this.machineId, text);
      }
    } catch (e) {
      this.controls.handleError(e);
      return;
    }
    let circuit = get_logic_circuit(this.machineId);
    this.view.draw(circuit);
  }

  step() {
    if (this.machineId == undefined) {
      this.controls.handleError("Please load code");
      return;
    }

    console.log("step");

    const inputs = this.view.get_inputs();

    try {
      step_logic_circuit(this.machineId, inputs);
    } catch (e) {
      this.controls.handleError(e);
      return;
    }

    let circuit = get_logic_circuit(this.machineId);
    this.view.draw(circuit);
  }
}
// ---- view models class end ----

// ---- view class ----
// use SVG.js for drawing
export class LogicCircuitView {
  inpins_state = null;

  constructor(viewId, default_placement) {
    // set default placement
    this.default_placement = default_placement;

    // setting for SVG.js
    this.viewId = viewId;
    this.SVG_elm = SVG().addTo(`#${viewId}`).size('100%', '100%').viewbox(0, 0, 800, 600);
    // set width and height
    this.SVG_elm.attr('style', 'border: 1px solid black;');
    this.group = this.SVG_elm.group();
  }

  get_inputs() {
    if (this.inpins_state == null) {
      // empty array
      return [];
    } else {
      // inpins_state to array
      const entries = this.inpins_state.entries();
      const arr = Array.from(entries, ([name, state]) => {
        console.log("get_inputs", name, state);
        return new PinWeb(name, state);
      });
      return arr;
    }
  }

  draw(circuit) {
    this.group.clear();
    const inpins = circuit.inpins;
    if (this.inpins_state == null) {
      this.inpins_state = new Map();
      for (let i = 0; i < inpins.length; i++) {
        this.inpins_state.set(inpins[i].name, false);
      }
      console.log("this.inpins_state", this.inpins_state);
    }
    const otpins = circuit.otpins;
    const boxes = circuit.boxes;
    const edges = circuit.edges;

    const map_pos = new Map();

    // draw boxes
    for (let i = 0; i < boxes.length; i++) {
      const name = boxes[i].name;
      const kind = boxes[i].kind;
      const inpins = boxes[i].inpins;
      const otpins = boxes[i].otpins;
      const state = boxes[i].state;
      console.log("draw boxes", name, kind, inpins, otpins, state);

      map_pos.set(name, i);

      // Create a group for each input pin
      let group = this.group.group();

      let is_gate = (kind == "_AND_" || kind == "_OR_" || kind == "_NOT_" || kind == "_BR_" || kind == "_CST_" || kind == "_DLY_" || kind == "END");

      // Draw a rectangle for the input pin
      group.rect(50, 30) // width: 50, height: 30
        .fill("white")
        .stroke(is_gate ? (state ? "green" : "red") : "black") // Green for "T", red for "F"

      // Add the name of the input pin as text
      group.text(kind)
        .font({ size: 12, fill: "black" })
        .move(0, 10);

      group.move(i * 70 + 5, 200 - 15);
      enableDrag(group);
    }

    console.log("map_pos", map_pos);

    let i = 0;
    // draw inpins use inpins_state instead of inpins
    for (const [name, state] of this.inpins_state) {
      console.log("draw inpins", name, state);

      // Create a group for each input pin
      let group = this.group.group();

      // Draw a rectangle for the input pin
      group.rect(50, 30) // width: 50, height: 30
        .fill("white")
        .stroke(state ? "green" : "red") // Green for "T", red for "F"
        ;

      // Add the name of the input pin as text
      group.text(name)
        .font({ size: 12, fill: "black" }) // Text styling
        .move(0, 10);

      // toggle inpins[name] if clicked
      group.on('click', () => {
        console.log("click inpins", name);
        const state = this.inpins_state.get(name);
        this.inpins_state.set(name, !state);
        this.draw(circuit);
      });

      group.move(i * 40 + 5, 15);
      i++;
    }

    // draw otpins
    for (let i = 0; i < otpins.length; i++) {
      console.log("draw otpins", otpins[i].name, otpins[i].state);
      const name = otpins[i].name;
      const state = otpins[i].state;

      // Create a group for each input pin
      let group = this.group.group();

      // Draw a rectangle for the input pin
      group.rect(50, 30) // width: 50, height: 30
        .fill("white")
        .stroke(state ? "green" : "red"); // Green for "T", red for "F"

      // Add the name of the input pin as text
      group.text(name)
        .font({ size: 12, fill: "black" })
        .move(0, 10);

      group.move(i * 40 + 5, 400 - 15);
      enableDrag(group);
    }

    // draw edges
    for (let i = 0; i < edges.length; i++) {
      const v_from = edges[i].from;
      const v_from_num = map_pos.get(v_from);
      const otpin = edges[i].otpin;
      const v_to = edges[i].to;
      const v_to_num = map_pos.get(v_to);
      const inpin = edges[i].inpin;
      console.log("draw edges", v_from, v_from_num, otpin, v_to, v_to_num, inpin);

      let group = this.group.group();

      group.line(x1, y1, x2, y2)

      map_pos[v_from]
    }
  }
}
// ---- view class end ----

// ---- drag SVG element ----
/**
 * SVG.js 要素をドラッグ＆ドロップ可能にする
 * @param {SVG.Element} node  ― SVG.js が返す要素（例: draw.rect(...), draw.circle(...))
 * @param {Object}      opts ― オプション
 *   opts.onStart(dx,dy,e)  : ドラッグ開始時コールバック
 *   opts.onMove(x,y,e)     : 移動中コールバック
 *   opts.onEnd(e)          : ドラッグ終了時コールバック
 */
function enableDrag(node, opts = {}) {
  let startX, startY;     // (SVG 座標) 押した瞬間の要素左上
  let pointerStartX, pointerStartY; // (screen 座標) 押した瞬間のポインタ位置
  let dragging = false;

  // --- helper ---
  function svgPoint(evt) {
    // page座標→SVG座標変換 (viewBox, transform 対応)
    const pt = node.root().node.createSVGPoint();
    pt.x = evt.clientX; pt.y = evt.clientY;
    return pt.matrixTransform(node.root().node.getScreenCTM().inverse());
  }

  // --- ハンドラ ---
  function pointerDown(evt) {
    evt.preventDefault();
    const p = svgPoint(evt);
    pointerStartX = evt.clientX;
    pointerStartY = evt.clientY;
    startX = node.x();          // 現在位置を記録
    startY = node.y();
    dragging = true;
    node.attr('pointer-events', 'none');   // 自分を無効化して下をクリックしない
    opts.onStart?.(startX, startY, evt);
    window.addEventListener('pointermove', pointerMove);
    window.addEventListener('pointerup', pointerUp, { once: true });
  }

  function pointerMove(evt) {
    if (!dragging) return;
    const dx = evt.clientX - pointerStartX;
    const dy = evt.clientY - pointerStartY;
    node.move(startX + dx, startY + dy);
    opts.onMove?.(startX + dx, startY + dy, evt);
  }

  function pointerUp(evt) {
    dragging = false;
    node.attr('pointer-events', 'auto');   // 再びクリック可
    opts.onEnd?.(evt);
    window.removeEventListener('pointermove', pointerMove);
  }

  // --- 監視開始 ---
  node.on('pointerdown', pointerDown);
}
// ---- drag SVG element end ----