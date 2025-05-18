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
  viewId = null;
  default_placement = null;
  machineId = undefined;

  constructor(codeResource, controls, viewId, default_placement) {
    // control: UserControls
    this.codeResource = codeResource;
    this.default_placement = default_placement;
    this.viewId = viewId;
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
    }
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

    this.view.reset();
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
  placement_box = new Map();
  placement_pin = new Map();
  edges_drawn = [];

  constructor(viewId, default_placement) {
    // set default placement
    // map of string -> SV.js's group, contains position of inpins, otpins and boxes
    if (default_placement != null) {
      this.default_placement = default_placement;
    };

    // setting for SVG.js
    this.viewId = viewId;
    this.SVG_elm = SVG().addTo(`#${viewId}`).size('100%', '100%').viewbox(0, 0, 800, 600);
    // set width and height
    this.SVG_elm.attr('style', 'border: 1px solid black;');
    this.group = this.SVG_elm.group();
  }

  reset() {
    // reset SVG part
    this.SVG_elm.clear();
    this.SVG_elm.attr('style', 'border: 1px solid black;');
    this.group = this.SVG_elm.group();

    this.edges_drawn = [];
    this.inpins_state = null;
    this.placement_box = new Map();

    for (const [name, pos] of this.default_placement) {
      // set the box position if not exists
      if (!this.placement_box.has(name)) {
        let group = this.group.group();
        let { draw, box } = drawBox(group, name, pos);
        this.placement_box.set(name, box);
        box.stroke("black");
        enableDrag(group, {
          onMove: () => {
            for (const edge of this.edges_drawn) {
              console.log("update edge", edge, name);
              if (edge.from_name === name || edge.to_name === name) {
                const from = this.placement_box.get(edge.from_name);
                const to = this.placement_box.get(edge.to_name);
                edge.line.plot(from.cx(), from.cy(), to.cx(), to.cy());
              }
            }
          }
        });
      }
    }
    this.placement_pin = new Map();
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

  redraw_inpins() {
    // redraw inpins
    // draw inpins use inpins_state instead of inpins
    for (const [name, state] of this.inpins_state) {
      console.log("redraw inpins", name, state);
      // get the rect from the placement
      let rect = this.placement_pin.get(name);
      rect.stroke(state ? "green" : "red"); // Green for "T", red for "F"
    }
  }

  draw(circuit) {
    // console.log("placement", this.placement);

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

    // draw boxes
    for (let i = 0; i < boxes.length; i++) {
      const name = boxes[i].name;
      const kind = boxes[i].kind;
      const inpins = boxes[i].inpins;
      const otpins = boxes[i].otpins;
      const state = boxes[i].state;
      console.log("draw boxes", name, kind, inpins, otpins, state);

      let rect;
      // set the box position if not exists
      if (!this.placement_box.has(name)) {
        let group = this.group.group();
        let { draw, box } = drawBox(group, name, { x: (i % 8) * 70 + 5, y: Math.floor(i / 8) * 50 + 150 });

        this.placement_box.set(name, box);
        rect = box;
        enableDrag(group, {
          onMove: () => {
            for (const edge of this.edges_drawn) {
              console.log("update edge", edge, name);
              if (edge.from_name === name || edge.to_name === name) {
                const from = this.placement_box.get(edge.from_name);
                const to = this.placement_box.get(edge.to_name);
                edge.line.plot(from.cx(), from.cy(), to.cx(), to.cy());
              }
            }
          }
        });
      } else {
        // get the rect from the placement
        rect = this.placement_box.get(name);
      }
      rect.stroke(is_gate(kind) ? (state ? "green" : "red") : "black"); // Green for "T", red for "F"
    }

    let i = 0;
    // draw inpins use inpins_state instead of inpins
    for (const [name, state] of this.inpins_state) {
      console.log("draw inpins", name, state);

      let rect;
      // set the circle position if not exists
      if (!this.placement_pin.has(name)) {
        let group = this.group.group();
        let { draw, circle } = drawCircle(group, name, { x: 0, y: 0 });
        console.log(circle);
        draw.move(i * 40 + 5, 15);

        circle.on('click', () => {
          console.log("click inpins", name);
          const state = this.inpins_state.get(name);
          this.inpins_state.set(name, !state);
          this.redraw_inpins();
        });

        this.placement_pin.set(name, circle);
        rect = circle;
        // toggle inpins[name] if clicked
      } else {
        // get the rect from the placement
        rect = this.placement_pin.get(name);
      }
      rect.stroke(state ? "green" : "red"); // Green for "T", red for "F"

      i++;
    }

    // draw otpins
    for (let i = 0; i < otpins.length; i++) {
      const name = otpins[i].name;
      const state = otpins[i].state;
      console.log("draw otpins", name, state);

      let rect;
      // set the circle position if not exists
      if (!this.placement_pin.has(name)) {
        let group = this.group.group();
        let { draw, circle } = drawCircle(group, name, { x: i * 40 + 5, y: 400 - 15 });
        this.placement_pin.set(name, circle);
        rect = circle;
      } else {
        // get the rect from the placement
        rect = this.placement_pin.get(name);
      }
      rect.stroke(state ? "green" : "red"); // Green for "T", red for "F"
    }

    // remove all edges (it's ok to remove all edges and draw again)
    this.edges_drawn.forEach(edge => {
      edge.line.remove();
    });
    this.edges_drawn = [];

    // draw edges between boxes
    for (let i = 0; i < edges.length; i++) {
      const from = edges[i].from;
      const v_from_name = from.name;
      const v_from = this.placement_box.get(v_from_name);
      const to = edges[i].to;
      const v_to_name = to.name;
      const v_to = this.placement_box.get(v_to_name);

      // console.log("draw edges", i, from, v_from_name, v_from, to, v_to_name, v_to);

      const line = lineFromPoints(this.group, { x: v_from.cx(), y: v_from.cy() }, { x: v_to.cx(), y: v_to.cy() });
      this.edges_drawn.push({ from_name: v_from_name, to_name: v_to_name, line });
    }

    // draw edges from inpins to boxes
    for (let i = 0; i < inpins.length; i++) {
      const v_from_name = inpins[i].name;
      const v_from = this.placement_pin.get(v_from_name);
      const to = circuit.get_inpins_map(v_from_name);
      if (to == null) {
        return;
      }
      const v_to_name = to.name;
      const v_to = this.placement_box.get(v_to_name);
      // console.log("draw edges (inpin)", v_from_name, v_from, v_to_name, v_to);
      const line = lineFromPoints(this.group, { x: v_from.cx(), y: v_from.cy() }, { x: v_to.cx(), y: v_to.cy() });
      this.edges_drawn.push({ from_name: v_from_name, to_name: v_to_name, line });
    }

    // draw edges from boxes to otpins
    for (let i = 0; i < otpins.length; i++) {
      const v_to_name = otpins[i].name;
      const v_to = this.placement_pin.get(v_to_name);
      const from = circuit.get_otpins_map(v_to_name);
      if (from == null) {
        return;
      }
      const v_from_name = from.name;
      const v_from = this.placement_box.get(v_from_name);
      // console.log("draw edges (otpin)", v_from_name, v_from, v_to_name, v_to);
      let line = lineFromPoints(this.group, { x: v_from.cx(), y: v_from.cy() }, { x: v_to.cx(), y: v_to.cy() });
      this.edges_drawn.push({ from_name: v_from_name, to_name: v_to_name, line });
    }

    console.log("draw placement", this.placement);
  }
}
// ---- view class end ----

// ---- helper functions ----
// draw a arrow line from a to b
function lineFromPoints(draw, a, b) {
  const line = draw.line(a.x, a.y, b.x, b.y).stroke({ color: 'black', width: 1, dasharray: '5,5' });
  const arrow = draw.marker(10, 10, (add) => {
    add.polyline('0,0 10,5 0,10 2,5').fill('black');
  });
  line.marker('end', arrow);
  return line;
}

// pos: center of box
// draw in a `draw` group
function drawBox(draw, text, a) {
  const box = draw.rect(50, 30).fill('white').stroke('black');
  draw.text(text).font({ size: 12, fill: 'black' }).move(5, 5);
  draw.move(a.x, a.y);
  return { draw, box };
}

function drawCircle(draw, text, a) {
  const circle = draw.circle(30).fill('white').stroke('black');
  draw.text(text).font({ size: 12, fill: 'black' }).move(5, 5);
  draw.move(a.x, a.y);
  return { draw, circle };
}

function is_gate(kind) {
  return (kind == "_AND_" || kind == "_OR_" || kind == "_NOT_" || kind == "_BR_" || kind == "_CST_" || kind == "_DLY_" || kind == "END");
}
// ---- helper functions end ----

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