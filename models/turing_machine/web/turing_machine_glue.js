import init, { CodeEntry, TapeForWeb, set_turing_machine, get_accepted_state, get_code, get_initial_state, get_now_state, get_now_tape, new_turing_machine, parse_code, parse_tape, next_direction, step_machine } from "./pkg/turing_machine_web.js";

export async function load() {
    await init();
}

export const ready = new Promise(resolve => {
    document.addEventListener("wasm-ready", resolve);
});

let TAPE_CELL_WIDTH = 20;
let TAPE_CELL_MARGIN = 5;
let TAPE_Y_CENTER = 15;

export function turing_machine_init(code_input, tape_input, load_code, load_tape, start_button, code_view, tape_view, step_button) {

    // html table element for code
    let table = document.createElement("table");
    table.innerHTML = ""; // Clear the table

    // add header to table
    let thead = table.createTHead();
    let headerRow = thead.insertRow();
    headerRow.insertCell().innerText = "now sign";
    headerRow.insertCell().innerText = "now state";
    headerRow.insertCell().innerText = "next sign";
    headerRow.insertCell().innerText = "next state";
    headerRow.insertCell().innerText = "direction";

    let code_table = document.getElementById(code_view);
    code_table.appendChild(table);

    // remember if or not of turing machine
    // if id === undefined => turing machine is not initialized
    let id;

    // :Code
    let code;
    // now state
    let now_state;

    // set value `code` and `now_state` if clicked
    document.getElementById(load_code)?.addEventListener("click", () => {
        console.log("load code");
        // get code text from textarea of id = code_input
        const code_text = document.getElementById(code_input)?.value;
        if (!code_text) {
            alert("Please write code");
            return;
        }
        console.log(code_text);

        try {
            code = parse_code(code_text);
            console.log(code);
        } catch (e) {
            alert(`${e}`);
            return;
        }

        now_state = code.init_state;
        drawTable(code.code, table);
    });

    // TapeForWeb
    let tape;

    // set value `tape` if clicked
    document.getElementById(load_tape)?.addEventListener("click", () => {
        console.log("load tape");
        // get tape text from textarea of id = tape_input
        const tape_text = document.getElementById(tape_input)?.value;
        if (!tape_text) {
            alert("Please write tape");
            return;
        }
        console.log(tape_text);

        try {
            tape = parse_tape(tape_text);
        } catch (e) {
            alert("Invalid tape format. Please check your tape.");
            return;
        }
        drawStateSVG(tape, now_state, cellGroupTape);

    });

    // SVG element for tape
    let drawtape = SVG().addTo(`#${tape_view}`).viewbox(-100, 0, 200, 40);
    let cellGroupTape = drawtape.group();

    // if code and tape is loaded
    // set turing machine
    document.getElementById(start_button)?.addEventListener("click", () => {
        console.log("start button clicked");

        // check if code and tape is loaded
        if (!code) {
            alert("Please load code first.");
            return;
        }
        if (!tape) {
            alert("Please load tape first.");
            return;
        }

        console.log(id);

        // set turing machine
        if (id === undefined) {
            console.log("new turing machine");
            id = new_turing_machine(code, tape);
        } else {
            console.log("set turing machine");
            set_turing_machine(id, code, tape);
        }

        console.log(id);

        // update value
        tape = get_now_tape(id);
        now_state = get_now_state(id);
        drawStateSVG(tape, now_state, cellGroupTape);
    });

    // step button
    document.getElementById(step_button)?.addEventListener("click", () => {
        console.log("step button clicked");
        if (id === undefined) {
            alert("Please load code and tape first.");
            return;
        }
        try {
            let next_direction_of_tape = next_direction(id);
        } catch {
            alert("no step");
            return;
        }
        animateTape(next_direction_of_tape, cellGroupTape, () => {
            // step machine
            step_machine(id);
            // update value
            tape = get_now_tape(id);
            now_state = get_now_state(id);
        });
    });
}

function drawTable(codeentryarr, table) {
    // Find all <tbody> elements
    const old_tbody = table.getElementsByTagName("tbody");
    // Remove all <tbody> elements
    while (old_tbody.length > 0) {
        table.removeChild(old_tbody[0]);
    }

    // Create a new <tbody> element
    const new_tbody = table.createTBody();

    // Create a new row for each entry in the code
    codeentryarr.forEach(entry => {
        console.log(entry);
        const row = new_tbody.insertRow();
        row.insertCell().innerText = entry.key_sign;
        row.insertCell().innerText = entry.key_state;
        row.insertCell().innerText = entry.next_sign;
        row.insertCell().innerText = entry.next_state;
        row.insertCell().innerText = entry.direction;
    });
}

function drawStateSVG(tape, state, cellGroup) {
    // Clear the existing SVG elements in the group
    cellGroup.clear();

    function drawCell(i, text, b) {
        const x = i * (TAPE_CELL_WIDTH + TAPE_CELL_MARGIN);
        const color = b ? "#f00" : "#0f0";
        cellGroup
            .rect(TAPE_CELL_WIDTH, TAPE_CELL_WIDTH)
            .center(x, TAPE_Y_CENTER)
            .fill("#fff")
            .stroke({ color: color, width: 1 });
        cellGroup
            .text(text)
            .move(x, TAPE_Y_CENTER - 10)
            .font({
                size: 5,
                anchor: "middle",
            });
    }

    // Draw the left part of the tape
    for (let i = 0; i < tape.left.length; i++) {
        drawCell(i - tape.left.length, tape.left[i], false);
    }

    // Draw the head
    drawCell(0, tape.head, true);

    // Draw the right part of the tape
    for (let i = 0; i < tape.right.length; i++) {
        drawCell(i + 1, tape.right[i], false);
    }

    // Draw the state under the head if is not falsy
    if (state) {
        cellGroup
            .text(state)
            .move(0, TAPE_Y_CENTER + 5)
            .font({
                size: 5,
                anchor: "middle",
            });
    }
}

function animateTape(direction, cellGroup, callback) {
    let delta = direction === "right" ? -(CELL_WIDTH + MARGIN) : (CELL_WIDTH + MARGIN);
    cellGroup.animate(200).dx(delta).after(() => {
        callback();
        cellGroup.dx(0);
    });
}
