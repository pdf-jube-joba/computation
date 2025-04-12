import init, { new_while_machine, set_while_machine, step_while_machine, get_code, get_current_line, get_env } from "./pkg/while_minus_lang_web.js";

// ---- wasm module glue code ----

export async function load() {
    await init();
}

// Use this function when you want to load the wasm module
export const ready = new Promise(resolve => {
    document.addEventListener("wasm-ready", resolve);
});

// ---- wasm module glue code end ----
export class WhileMinusLangViewModel {
    code = null;
    env = null;
    machineId = undefined;

    constructor(codeResource, envResource, controls, viewId) {
        // control: UserControls
        this.codeResource = codeResource;
        this.envResource = envResource;
        this.view = new WhileMinusLangView(viewId);
        this.controls = controls;
        this.controls.setOnLoad(() => {
            this.loadCode();
            this.loadEnv();
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
        this.code = text;
        console.log("code", this.code);
    }

    loadEnv() {
        const text = this.envResource.getText();
        console.log("load env", text);
        if (!text) {
            this.controls.handleError("Please write env");
            return;
        };
        this.env = text;
        console.log("env", this.env);
    }

    start() {
        console.log("start");

        if (this.machineId === undefined) {
            console.log("new while machine");
            try {
                this.machineId = new_while_machine(this.code, this.env);
            } catch (e) {
                this.controls.handleError(e);
                return;
            }
        } else {
            console.log("set while machine", this.machineId);
            try {
                set_while_machine(this.machineId, this.code, this.env);
            } catch (e) {
                this.controls.handleError(e);
                return;
            }
        }
        console.log("machineId", this.machineId);

        this.view.update({ code: get_code(this.machineId), env: get_env(this.machineId), currentLine: get_current_line(this.machineId) });
    }

    step() {
        console.log("step");
        if (this.machineId === undefined) {
            this.controls.handleError("Please load code and env first");
            return;
        }
        try {
            step_while_machine(this.machineId);
            this.view.update({ code: get_code(this.machineId), env: get_env(this.machineId), currentLine: get_current_line(this.machineId) });
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
    }
}

export class WhileMinusLangView {
    constructor(container) {
        // --- initialize container
        this.container = typeof container === "string"
            ? document.getElementById(container)
            : container;
        this.container.innerHTML = ""; // clear

        // --- table for env
        this.codeTable = document.createElement("table");
        // add thead
        const thead = this.codeTable.createTHead();
        const row = thead.insertRow();
        row.insertCell().innerText = "variable";
        row.insertCell().innerText = "number";
        // add thead
        this.container.appendChild(this.codeTable);

        // --- div for code
        this.codePreview = document.createElement("code");
        this.codePreview.innerText = "";
        // add code div
        this.container.appendChild(this.codePreview);
    }

    update({ code, env, currentLine }) {
        this.drawCode(code, currentLine);
        this.drawEnv(env);
    }

    drawCode(codearr, currentLine) {
        // clear old code
        this.codePreview.innerText = "";

        // for each line in codearr, add line number + <pre> code </pre>
        codearr.forEach((line, index) => {
            const lineDiv = document.createElement("pre");
            lineDiv.className = "line";
            lineDiv.innerText = `${index + 1}: ${line}`;
            if (index === currentLine) {
                lineDiv.classList.add("current");
            }
            this.codePreview.appendChild(lineDiv);
        });
    }

    drawEnv(envarr) {
        // clear old env
        this.codeTable.innerHTML = "";

        // add thead
        const thead = this.codeTable.createTHead();
        const row = thead.insertRow();
        row.insertCell().innerText = "variable";
        row.insertCell().innerText = "number";
        // add thead
        this.container.appendChild(this.codeTable);

        // draw new env
        const newTbody = this.codeTable.createTBody();
        envarr.forEach((entry) => {
            const row = newTbody.insertRow();
            row.insertCell().innerText = entry.name;
            row.insertCell().innerText = entry.value;
        });
    }
}
