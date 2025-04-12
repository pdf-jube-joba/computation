import init, { new_while_machine, set_while_machine, get_code, get_current_line, get_env } from "./pkg/while_minus_lang_web.js";

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
    currentLine = null;

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
        try {
            this.code = text;
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
        console.log("code", this.code);
    }

    loadEnv() {
        const text = this.envResource.getText();
        console.log("load env", text);
        if (!text) {
            this.controls.handleError("Please write env");
            return;
        };
        try {
            this.env = text;
        } catch (e) {
            this.controls.handleError(e);
            return;
        }
        console.log("env", this.env);
    }

    start() {
        console.log("start");

        if (this.machineId === undefined) {
            console.log("new while machine", this.code, this.env);
            try {
                this.machineId = new_while_machine(this.code, this.env);
            } catch (e) {
                this.controls.handleError(e);
                return;
            }
        } else {
            try {
                set_while_machine(this.machineId, this.code, this.env);
            } catch (e) {
                this.controls.handleError(e);
                return;
            }
            console.log("machineId", this.machineId);
        }

        this.currentLine = get_current_line(this.machineId);
        this.env = get_env(this.machineId);
        this.view.update({ code: this.code, env: this.env, currentLine: this.currentLine });
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

        // --- pre for code
        this.codePre = document.createElement("pre");
        this.codePre.className = "code";
        this.codePre.innerText = "";
    }

    update({ code, env, currentLine }) {
        this.drawCode(code, currentLine);
        this.updateEnv(env);
    }

    drawCode(codearr, currentLine) {
        // clear old code
        this.codePre.innerText = "";
        // draw new code
        codearr.forEach((entry, index) => {
            const line = document.createElement("div");
            line.innerText = entry;
            if (index === currentLine) {
                line.className = "current-line";
            }
            this.codePre.appendChild(line);
        });
        this.container.appendChild(this.codePre);
    }

    drawEnv(envarr) {
        // clear old env
        this.codeTable.innerHTML = "";

        // draw new env
        const newTbody = this.codeTable.createTBody();
        envarr.forEach((entry) => {
            const row = newTbody.insertRow();
            row.insertCell().innerText = entry.variable;
            row.insertCell().innerText = entry.number;
        });
    }
}
