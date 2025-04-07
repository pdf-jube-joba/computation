import init, { CodeEntry, TapeForWeb, tape_left_index, tape_right_index, set_turing_machine, get_accepted_state, get_code, get_next_codeentry_index, get_initial_state, get_now_state, get_now_tape, new_turing_machine, parse_code, parse_tape, next_direction, step_machine } from "./pkg/turing_machine_web.js";

// ---- wasm module glue code ----

export async function load() {
    await init();
}

// Use this function when you want to load the wasm module
export const ready = new Promise(resolve => {
    document.addEventListener("wasm-ready", resolve);
});

// ---- wasm module glue code end ----

// ---- input resource class ----
// We can use this classes as text source

// class provide text source from given textarea
export class TextAreaSource {
    // constructor from textarea_id
    constructor(textarea_id) {
        this.textarea = document.getElementById(textarea_id);
    }

    // return text: string
    getText() {
        return this.textarea.value;
    }
}

// class provide text source from given text
export class TextDefinedSource {
    constructor(text) {
        this.text = text;
    }

    // return text: string
    getText() {
        return this.text;
    }
}

// ---- input resource class end ----

// ---- turing machine model class ----

export class TuringMachineViewModel {
    code = null;
    tape = null;
    machineId = undefined;
    currentState = null;

    constructor(codeResource, tapeResource, viewId, startButtonId, stepButtonId) {
        this.codeResource = codeResource;
        this.tapeResource = tapeResource;
        this.view = new TuringMachineView(viewId);
        this.startButton = document.getElementById(startButtonId);
        this.startButton.onclick = () => {
            this.loadCode();
            this.loadTape();
            this.start();
        };
        this.stepButton = document.getElementById(stepButtonId);
        this.stepButton.onclick = () => {
            this.step();
        };
    }

    loadCode() {
        console.log("load code");
        const text = this.codeResource.getText();
        if (!text) return alert("Please write code");
        try {
            this.code = parse_code(text);
            this.currentState = this.code.init_state;
        } catch (e) {
            alert(`${e}`);
            return;
        }
    }

    loadTape() {
        console.log("load tape");
        const text = this.tapeResource.getText();
        if (!text) return alert("Please write tape");
        try {
            this.tape = parse_tape(text);
        } catch (e) {
            alert(`${e}`);
            return;
        }
    }

    start() {
        console.log("start");
        if (!this.code || !this.tape) return alert("Please load code and tape");

        if (this.machineId === undefined) {
            this.machineId = new_turing_machine(this.code, this.tape);
        } else {
            set_turing_machine(this.machineId, this.code, this.tape);
        }

        this.tape = get_now_tape(this.machineId);
        this.currentState = get_now_state(this.machineId);
        this.view.update({
            tape: this.tape,
            state: this.currentState,
            code: get_code(this.machineId),
        });
    }

    step() {
        console.log("step");
        if (this.machineId === undefined) return alert("Please initialize first");
        let direction;
        try {
            direction = next_direction(this.machineId);
        } catch {
            alert("No step");
            return;
        }

        this.view.animateTape(direction, () => {
            step_machine(this.machineId);
            this.tape = get_now_tape(this.machineId);
            this.currentState = get_now_state(this.machineId);
            const index = ne
            this.view.update({
                tape: this.tape,
                state: this.currentState,
                code: get_code(this.machineId),
            });
        });
    }
}

export class TuringMachineView {
    constructor(container) {
        console.log("TuringMachineView");

        // --- initialize container
        this.container = typeof container === "string"
            ? document.getElementById(container)
            : container;
        this.container.innerHTML = ""; // clear

        // --- svg wrapper for tape
        const svgWrapper = SVG().addTo(this.container).viewbox(-100, 0, 200, 40);
        this.tapeGroup = svgWrapper.group();

        // layout / spacing config
        this.cellWidth = 20;
        this.cellMargin = 5;
        this.tapeYCenter = 15;

        // --- table for code
        this.codeTable = document.createElement("table");
        // add thead
        const thead = this.codeTable.createTHead();
        const row = thead.insertRow();
        row.insertCell().innerText = "key_sign";
        row.insertCell().innerText = "key_state";
        row.insertCell().innerText = "next_sign";
        row.insertCell().innerText = "next_state";
        row.insertCell().innerText = "direction";
        // add thead
        this.container.appendChild(this.codeTable);
    }

    update({ tape, state, code, index }) {
        this.drawCode(code);
        this.drawTape(tape, state);
        if (index !== undefined) {
            this.highlightCodeIndex(index);
        }
    }

    animateTape(direction, afterCallback) {
        console.log("animate tape", direction);
        const delta =
            direction === "right"
                ? (this.cellWidth + this.cellMargin)
                : -(this.cellWidth + this.cellMargin);
        this.tapeGroup.animate(200).dx(delta).after(() => {
            afterCallback();
            this.tapeGroup.dx(0);
        });
    }

    drawCode(codeEntryArr) {
        const oldTbody = this.codeTable.getElementsByTagName("tbody");
        while (oldTbody.length > 0) {
            this.codeTable.removeChild(oldTbody[0]);
        }

        const newTbody = this.codeTable.createTBody();
        codeEntryArr.forEach(entry => {
            const row = newTbody.insertRow();
            row.insertCell().innerText = entry.key_sign;
            row.insertCell().innerText = entry.key_state;
            row.insertCell().innerText = entry.next_sign;
            row.insertCell().innerText = entry.next_state;
            row.insertCell().innerText = entry.direction;
        });
    }

    highlightCodeIndex(index) {
        // get n-th tr element of codeTable in tbody
        const tbody = this.codeTable.getElementsByTagName("tbody")[0];
        const rows = tbody.getElementsByTagName("tr");
        for (let i = 0; i < rows.length; i++) {
            if (i === index) {
                rows[i].style.backgroundColor = "#f00";
            } else {
                rows[i].style.backgroundColor = "#fff";
            }
        }
    }

    drawTape(tape, state) {
        this.tapeGroup.clear();

        const drawCell = (i, text, isHead) => {
            const x = i * (this.cellWidth + this.cellMargin);
            const color = isHead ? "#f00" : "#0f0";
            this.tapeGroup
                .rect(this.cellWidth, this.cellWidth)
                .center(x, this.tapeYCenter)
                .fill("#fff")
                .stroke({ color: color, width: 1 });
            this.tapeGroup
                .text(text)
                .move(x, this.tapeYCenter - 10)
                .font({ size: 5, anchor: "middle" });
        };

        for (let i = 0; i < 4; i++) {
            drawCell(- i - 1, tape_left_index(tape, i), false);
        }

        drawCell(0, tape.head, true);

        for (let i = 0; i < 4; i++) {
            drawCell(i + 1, tape_right_index(tape, i), false);
        }

        if (state) {
            this.tapeGroup
                .text(state)
                .move(0, this.tapeYCenter + 5)
                .font({ size: 5, anchor: "middle" });
        }
    }
}

// ---- turing machine model class end ----
