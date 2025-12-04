// assets/renderers/counter.js
export function render(state, ctx, canvas, vm) {
  // state: current_machine が返した JS オブジェクト
  // ctx:   canvas.getContext("2d")
  // canvas: HTMLCanvasElement
  // vm:    ViewModel インスタンス（必要なら UI にアクセスできる）

  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const count = (state && typeof state.count === "number") ? state.count : 0;

  ctx.font = "20px sans-serif";
  ctx.textBaseline = "top";
  ctx.fillText(`count = ${count}`, 10, 10);
}
