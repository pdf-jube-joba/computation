import init, {
    TapeWeb,
    move_left, move_right, head, left, right, tape_parse,
} from "./pkg/turing_machine_web.js";

let CELL_WIDTH = 20;
let TAPE_Y = 20;

export async function tape_initialize(canvas_id, left_btn, right_btn) {
    await init();
    console.log("test");

    let tape = tape_parse("a, b, c, d", "-", "0, 1, 2");

    const draw = SVG().addTo(`#${canvas_id}`).viewbox(-100, 0, 200, 40);

    // fixed triangle on head position
    draw.polygon('0,20 10,0 -10,0')
        .fill('red')
        .id('head-triangle');

    // group for the tape cells
    let cellGroup = draw.group();

    function renderTape() {
        cellGroup.clear();

        let left_tape = left(tape);

        console.log(left_tape);

        let head_sign = head(tape);
        let right_tape = right(tape);

        // draw the head
        cellGroup.rect(CELL_WIDTH, 20)
            .fill('green')
            .move(0 - CELL_WIDTH / 2, TAPE_Y)
            .text("");

        // // for each symbol in left tape, draw a rectangle
        // left_tape.forEach((symbols, i) => {
        //     cellGroup.rect(CELL_WIDTH, 20)
        //         .fill('blue')
        //         .move(-CELL_WIDTH * (i + 1), TAPE_Y)
        //         .text(symbols);
        // });

        // // for each symbol in right tape, draw a rectangle
        // right_tape.forEach((symbols, i) => {
        //     cellGroup.rect(CELL_WIDTH, 20)
        //         .fill('blue')
        //         .move(CELL_WIDTH * i, TAPE_Y)
        //         .text(symbols);
        // });

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

