import init, { get_process, new_machine, parse_code, parse_vec, set_machine, step_process } from "./pkg/recursive_function_web.js";

// ---- wasm module glue code ----

export async function load() {
    await init();
}

// Use this function when you want to load the wasm module
export const ready = new Promise(resolve => {
    document.addEventListener("wasm-ready", resolve);
});

// ---- wasm module glue code end ----

// ---- view model class ---- 
export class RecursiveFunctionViewModel {
    process = null;
    tuple = null;
    machineId = undefined;

    constructor(codeResource, tupleResource, controls, viewId) {
        // control: UserControls
        this.codeResource = codeResource;
        this.tupleResource = tupleResource;
        this.view = new RecursiveFunctionView(viewId);
        this.controls = controls;
        this.controls.setOnLoad(() => {
            this.loadCode();
            this.loadTuple();
            this.start();
        });
        this.controls.setOnStep(() => {
            this.step();
        });
    }

    loadCode() {
        const text = this.codeResource.getText();
        console.log("load code", text);
        if (!text) {
            this.controls.handleError("Please write code");
            return;
        };
        try {
            this.function = parse_code(text);
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
        console.log("code", this.function.into_string);
    }

    loadTuple() {
        const text = this.tupleResource.getText();
        console.log("load tuple", text);
        if (!text) {
            this.controls.handleError("Please write tuple");
            return;
        };
        try {
            this.tuple = parse_vec(text);
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
        console.log("tuple", this.tuple.as_vec);
    }

    start() {
        console.log("start");
        try {
            if (this.machineId == undefined) {
                this.machineId = new_machine(this.function, this.tuple);
            } else {
                set_machine(this.machineId, this.function, this.tuple);
            }
        } catch (e) {
            this.controls.handleError(e);
            return;
        }

        console.log("machineId", this.machineId, "process", get_process(this.machineId));

        this.view.reset();
        this.view.addHistory(get_process(this.machineId));
    }

    step() {
        if (this.machineId == undefined) {
            this.controls.handleError("No machine loaded. Please load a term first.");
            return;
        }

        try {
            step_process(this.machineId);
            this.view.addHistory(get_process(this.machineId));
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
    }
}
// ---- view model class end ----

// ---- view class ----
export class RecursiveFunctionView {
    history = [];

    constructor(viewId) {
        this.viewId = viewId;
        this.container = document.getElementById(viewId);
        this.container.innerHTML = "";
    }

    reset() {
        this.history = [];
        this.render();
    }

    addHistory(process) {
        this.history.push(process);
        this.render();
    }

    render() {
        this.container.innerHTML = "";

        const processElement = document.createElement("div");

        this.history.forEach((process, index) => {
            const processDiv = document.createElement("span");
            processDiv.innerText = `Step ${index + 1}: ${process.into_string}`;
            processElement.appendChild(processDiv);
            processElement.appendChild(document.createElement("br"));
        });

        this.container.appendChild(processElement);
    }
}


