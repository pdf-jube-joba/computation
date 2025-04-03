import init, { new_tape, left, right, head, move_left, move_right } from "./pkg/test_global_tape.js";

export async function load() {
    await init();
}

let CELL_WIDTH = 20;
let MARGIN = 5;
let TAPE_Y = 15;

// get id of div, create a new tape, draw the tape with SVG
export function add_tape(canvas_id, left_str, head_str, right_str, left_btn, right_btn) {

    // create new tape from wasm and get id
    let id = new_tape(left_str, head_str, right_str);

    const draw = SVG().addTo(`#${canvas_id}`).viewbox(-100, 0, 200, 40);

    // test
    console.log("test");
    // draw.polygon('0,20 10,0 -10,0').fill('red');

    let cellGroup = draw.group();

    function renderTape() {
        cellGroup.clear();

        function cellwrite(i, text) {
            let x = i * (CELL_WIDTH + MARGIN);
            cellGroup.circle(1).center(x, TAPE_Y).fill("#111");
            cellGroup.rect(CELL_WIDTH, CELL_WIDTH).center(x, TAPE_Y).fill("#fff").stroke({ color: "#000", width: 1 });
            // text with center 
            cellGroup.text(text).move(x, TAPE_Y - 10).font({
                size: 5,
                anchor: 'middle',
            });
        }

        let left_tape = left(id);
        left_tape.forEach((symbol, i) => {
            cellwrite((i - 3), symbol);
        });

        cellwrite(0, head(id));

        let right_tape = right(id);
        right_tape.reverse();
        right_tape.forEach((symbol, i) => {
            cellwrite(i + 1, symbol);
        });
    }

    function animateTape(direction) {
        let delta = direction === "right" ? -CELL_WIDTH : CELL_WIDTH;
        cellGroup.animate(300).dx(delta).after(() => {
            renderTape();
            cellGroup.dx(0);
        });
    }

    document.getElementById(left_btn)?.addEventListener("click", () => {
        move_left(id);
        animateTape("left");
    });

    document.getElementById(right_btn)?.addEventListener("click", () => {
        move_right(id);
        animateTape("right");
    });

    renderTape();
}
