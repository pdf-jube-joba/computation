import init, { get_lambda_term, new_lambda_term, parse_lambda, set_lambda_term, step_lambda_term, get_marked_term } from "./pkg/lambda_calculus_web.js";

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

        this.view.reset();
        this.view.addTerm(this.term);
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
            console.log("step", this.term.into_string);
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
        this.view.addTerm(this.term);
        this.view.render();
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

    reset() {
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

        const codeElement = document.createElement("div");

        // div for each term in history
        this.history.forEach((term, index) => {
            try {
                const termElement = render_term_and_marked_term(term, index);
                codeElement.appendChild(termElement);
            } catch (e) {
                console.log(e);
                return;
            }
        });

        // add code div
        this.container.appendChild(codeElement);
    }

}

function render_term_and_marked_term(term, index) {
    const termElement = document.createElement("pre");
    // index
    termElement.appendChild(span_string("Step " + (index + 1) + ": ", "index"));
    // term
    termElement.appendChild(span_string(term.into_string, "term"));

    // marked term
    const markedTerm = get_marked_term(term);

    if (!markedTerm) {
        return termElement;
    }

    let breakline = document.createElement("br");
    termElement.appendChild(breakline);

    termElement.appendChild(coloring_marked_term(markedTerm));
    // return
    return termElement;
}

function coloring_term(term) {
    console.log("coloring_term", term.into_string);
    return span_string(term.into_string);
}

// input: ReduxmWasm
// return: html element with hightlighted term
function coloring_redux(term) {
    console.log("coloring_redux", "var", term.var_of_term, "body", term.body_of_term.into_string, "arg", term.arg_of_term.into_string);
    const termElement = document.createElement("span");
    termElement.className = "line";

    termElement.appendChild(span_string(`(\\${term.var_of_term}.`, "var_lambda"));

    termElement.appendChild(span_string(term.body_of_term.into_string + ") ", "body_lambda"));

    termElement.appendChild(span_string(term.arg_of_term.into_string, "arg_lambda"));

    return termElement;
}

// input: marked term
// return: html element with hightlighted term
function coloring_marked_term(term) {
    console.log("coloring_marked_term", term.into_string, "kind", term.kind);
    const kind = term.kind;

    if (kind == "redux") {
        return coloring_redux(term.as_redux);
    } else if (kind == "abstraction") {
        const termElement = document.createElement("span");
        termElement.appendChild(span_string(`\\${term.abs_var}.`));
        termElement.appendChild(coloring_marked_term(term.abs_body));
        return termElement;
    } else if (kind == "applicationL") {
        const termElement = document.createElement("span");

        termElement.appendChild(coloring_marked_term(term.app_left));
        termElement.appendChild(coloring_term(term.app_left_else));

        return termElement;
    } else if (kind == "applicationR") {
        const termElement = document.createElement("span");

        termElement.appendChild(coloring_term(term.app_right_else));
        termElement.appendChild(coloring_marked_term(term.app_right));

        return termElement;
    } else {
        console.log("term kind not supported", kind);
    }

}

function span_string(string, className = null) {
    const span = document.createElement("span");
    if (className) {
        span.className = className;
    }
    span.innerText = string;
    return span;
}
