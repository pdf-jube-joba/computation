// assets/renderers/example.js
// Renderer class with drawState/drawOutput
export class Renderer {
  constructor(vm) {
    this.vm = vm;
  }

  drawState(state, ctx, canvas) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const raw = state instanceof Map ? state.get("count") : state && state.count;
    const count = typeof raw === "number" ? raw : Number(raw) || 0;

    ctx.font = "20px sans-serif";
    ctx.textBaseline = "top";
    ctx.fillText(`count = ${count}`, 10, 10);
  }

  drawOutput(output, ctx, canvas) {
    if (!output) return;
    ctx.font = "16px sans-serif";
    ctx.textBaseline = "top";
    ctx.fillText(`output: ${output}`, 10, 40);
  }
}
