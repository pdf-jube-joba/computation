import init from "./pkg/logic_circuit_web.js";

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
  circuit = null;
  machineId = undefined;

  constructor(codeResource, controls, viewId) {
    // control: UserControls
    this.codeResource = codeResource;
    this.view = new LogicCircuitView(viewId);
    this.controls = controls;
    this.controls.setOnLoad(() => {
      this.loadCode();
      this.start();
    });
    this.controls.setOnStep(() => {
      this.step();
    });
  }
}
// ---- view models class end ----

// ---- view class ----
// use SVG.js for drawing
export class LogicCircuitView {

  constructor(viewId, placement) {
    this.viewId = viewId;
    this.SVG_elm = SVG().addTo(`#${viewId}`).size('100%', '100%').viewbox(0, 0, 800, 600);
    // set width and height
    this.SVG_elm.attr('style', 'border: 1px solid black;');
    this.group = this.SVG_elm.group();

    this.draw();
  }

  draw(circuit) {
    // test code for drag rect
    this.group.clear();
    const rect = this.group.rect(100, 100).attr({ fill: '#f06' });
    enableDrag(rect);
    const rect2 = this.group.rect(100, 100).attr({ fill: '#0f6' });
    enableDrag(rect2, {
      onStart: (x, y) => {
        console.log("drag start", x, y);
      },
      onMove: (x, y) => {
        console.log("drag move", x, y);
      },
      onEnd: (e) => {
        console.log("drag end", e);
      }
    });
    rect2.move(200, 200);
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