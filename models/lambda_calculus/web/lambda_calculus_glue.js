import init, { get_lambda_term, new_lambda_term, parse_lambda, set_lambda_term, step_lambda_term } from "./pkg/lambda_calculus_web.js";

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

export class LambdaCalculusViewModel {
    term = null;
    machineId = undefined;

    constructor(codeResource, controls, viewId) {
        // control: UserControls
        this.codeResource = codeResource;
        this.view = new LambdaCalculusView(viewId);
        this.controls = controls;
        this.controls.setOnLoad(() => {
            this.loadCode();
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
            this.term = parse_lambda(text);
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
        console.log("code", this.term.into_string);
    }

    start() {
        console.log("start");
        // try {
            if (this.machineId == undefined) {
                this.machineId = new_lambda_term(this.term);
            } else {
                set_lambda_term(this.machineId, this.term);
            }
        // } catch (e) {
        //     this.controls.handleError(e);
        //     return;
        // }

        console.log("machineId", this.machineId, "term", this.term.into_string);

        this.view.reset(this.term);
        this.view.render();
    }

    step() {
        if (!this.term) {
            this.controls.handleError("No term loaded. Please load a term first.");
            return;
        }

        try {
            step_lambda_term(this.machineId, this.term);
            this.term = get_lambda_term(this.machineId);
            this.view.addTerm(this.term);
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
    }
}

export class LambdaCalculusView {
    history = [];

    constructor(viewId) {
        this.viewId = viewId;
        this.container = document.getElementById(viewId);
        if (!this.container) {
            throw new Error(`Element with id '${viewId}' not found`);
        }
    }

    reset(start_term) {
        this.start_term = start_term;
        this.history = [];
    }

    addTerm(term) {
        this.history.push(term);
        this.render();
    }

    // render the terms form history
    render() {
        // clear the container
        this.container.innerHTML = "";

        // 
        const codeElement = document.createElement("div");

        // div for start_term
        const startTermElement = document.createElement("div");
        startTermElement.innerText = `Start term: ${this.start_term.into_string}`;
        codeElement.appendChild(startTermElement);

        // div for each term in history
        this.history.forEach((term, index) => {
            const termElement = document.createElement("div");
            termElement.innerText = `Step ${index + 1}: ${term.into_string}`;
            codeElement.appendChild(termElement);
        });

        // add code div
        this.container.appendChild(codeElement);
    }
}