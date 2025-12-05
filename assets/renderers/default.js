// assets/renderers/counter.js
export function render(state, ctx, canvas, vm) {
  // state: current_machine が返した JS オブジェクト
  // ctx:   canvas.getContext("2d")
  // canvas: HTMLCanvasElement
  // vm:    ViewModel インスタンス（必要なら UI にアクセスできる）

  
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  console.log("default renderer:", state);
  const raw = state instanceof Map ? state.get("count") : state && state.count;
  const count = typeof raw === "number" ? raw : Number(raw) || 0;
  console.log("count =", count);

  ctx.font = "20px sans-serif";
  ctx.textBaseline = "top";
  ctx.fillText(`count = ${count}`, 10, 10);
}
