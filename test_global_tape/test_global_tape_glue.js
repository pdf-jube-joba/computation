import init, { new_tape, left, right, head, move_left, move_right } from "./pkg/test_global_tape.js";

export async function load() {
    await init();
}

export const ready = new Promise(resolve => {
    document.addEventListener("wasm-ready", resolve);
});

let CELL_WIDTH = 20;
let MARGIN = 5;
let TAPE_Y = 15;

// get id of div, create a new tape, draw the tape with SVG
export function add_tape(canvas_id, left_str, head_str, right_str, left_btn, right_btn) {

    // create new tape from wasm and get id
    let id = new_tape(left_str, head_str, right_str);

    const draw = SVG().addTo(`#${canvas_id}`).viewbox(-100, 0, 200, 40);

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
        for (let i = 0; i < 3; i++) {
            let j = left_tape.length - 3 + i;
            if (j < 0) {
                cellwrite(i - 3, " ");
            } else {
                cellwrite(i - 3, left_tape[j]);
            }

        }

        cellwrite(0, head(id));

        let right_tape = right(id);
        for (let i = 0; i < 3; i++) {
            let j = right_tape.length - 1 - i;
            if (j < 0) {
                cellwrite(i + 1, " ");
            } else {
                cellwrite(i + 1, right_tape[j]);
            }
        }
    }

    function animateTape(direction) {
        let delta = direction === "right" ? -(CELL_WIDTH + MARGIN) : (CELL_WIDTH + MARGIN);
        cellGroup.animate(200).dx(delta).after(() => {
            renderTape();
            cellGroup.dx(0);
        });
    }

    document.getElementById(left_btn)?.addEventListener("click", () => {
        animateTape("left");
        move_left(id);
    });

    document.getElementById(right_btn)?.addEventListener("click", () => {
        animateTape("right");
        move_right(id);
    });

    renderTape();
}
