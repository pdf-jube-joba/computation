import init, {
    tape_parse,
    move_left,
    move_right,
    head,
    left,
    right,
} from "./pkg/turing_machine_web.js";

// SVG描画用の関数
export async function load() {
    const draw = SVG().addTo("#canvas").size(400, 100);
    draw.text("hello").move(20, 20);
}
