// SVG描画用の関数
export async function load(parse) {
    const result = parse(); // wasm関数呼び出し

    const draw = SVG().addTo("#canvas").size(400, 100);
    draw.text(result).move(20, 20);
}