import init, {
    TapeWeb,
    move_left, move_right, head, left, right, tape_parse,
} from "./pkg/turing_machine_web.js";

let CELL_WIDTH = 60;
let TAPE_Y = 50;

export async function tape_initialize(canvas_id, left_btn, right_btn) {
    await init();
    console.log("test");

    let tape = tape_parse("a, b, c, d", "-", "0, 1, 2");

    const draw = SVG().addTo(`#${canvas_id}`).viewbox(-100, 0, 200, 40);

    // 中央固定のヘッド三角形
    draw.polygon('0,20 10,0 -10,0')
        .fill('red')
        .id('head-triangle');

    let cellGroup = draw.group();

    function renderTape() {
        cellGroup.clear();

        let flattened = [...left(tape), head(tape), ...right(tape)];

        console.log(flattened);

        flattened.forEach((symbol, i) => {
            let x = i * CELL_WIDTH;
            cellGroup.rect(CELL_WIDTH - 4, 40).move(x, TAPE_Y).fill('#fff').stroke({ width: 1 });
            cellGroup.text(symbol).move(x + 20, TAPE_Y + 10).font({ size: 20 });
        });

        let headIndex = left(tape).length;
        cellGroup.translate(centerX - headIndex * CELL_WIDTH - CELL_WIDTH / 2, 0);
    }

    function animateTape(direction) {
        let delta = direction === "right" ? -CELL_WIDTH : CELL_WIDTH;
        cellGroup.animate(300).dx(delta).after(() => {
            renderTape();
            cellGroup.dx(0);
        });
    }

    document.getElementById(left_btn)?.addEventListener("click", () => {
        move_left(tape);
        animateTape("left");
    });

    document.getElementById(right_btn)?.addEventListener("click", () => {
        move_right(tape);
        animateTape("right");
    });

    renderTape();
}

