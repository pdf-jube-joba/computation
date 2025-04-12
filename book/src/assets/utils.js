// ---- input resource class ----
// We can use this classes as text source

// class provide text source from given textarea
export class TextAreaSource {
    is_ready = false;

    // constructor from textarea_id
    constructor(textarea_id) {
        this.textarea = document.getElementById(textarea_id);
        // prevent propagation of keyevent
        // to chapter navigation
        // next page by rightarrow key
        // prev page by leftarrow key
        this.textarea.addEventListener("keydown", (e) => {
            if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
                e.stopPropagation();
            }
        });
    }

    // return text: string
    getText() {
        return this.textarea.value;
    }
}

// class provide text source from given text
export class TextDefinedSource {
    is_ready = true;

    // constructor from text
    constructor(text) {
        this.text = text;
    }

    // return text: string
    getText() {
        return this.text;
    }
}

// ---- input resource class end ----

// ---- control class ----
// class provide controls of the machine
// - load
// - run (one step)

// controls by user
// which calls
// (user can auto_step and auto_stop)
export class UserControls {
    // interval time in ms
    time_interval = 300;

    constructor(control_id) {
        const container = document.getElementById(control_id);
        if (!container) {
            throw new Error(`Element with id '${control_id}' not found`);
        }
        this.container = container;

        // create load button element under control div
        this.load_btn = document.createElement("button");
        this.load_btn.innerText = "Load";
        this.load_btn.id = "load_btn";
        this.container.appendChild(this.load_btn);

        // create step button element under control div
        this.step_btn = document.createElement("button");
        this.step_btn.innerText = "Step";
        this.step_btn.id = "step_btn";
        this.container.appendChild(this.step_btn);

        // create auto step button element under control div
        this.auto_btn = document.createElement("button");
        this.auto_btn.innerText = "Auto: off";
        this.auto_btn.id = "auto_step_btn";
        this.container.appendChild(this.auto_btn);

        this.step_fn = () => { };
        this.load_fn = () => { };

        // set default value
        this.auto_mode = false;
        this.auto_interval = null;

        this.step_btn?.addEventListener("click", () => this.step_fn());
        this.load_btn?.addEventListener("click", () => this.load_fn());

        this.auto_btn.addEventListener("click", () => this.toggleAuto());

        // create error handler
        this.error_handler = (e) => {
            alert(`Error: ${e}`);
        };
    }

    toggleAuto() {
        if (!this.auto_mode) {
            this.auto_mode = true;
            this.auto_btn.textContent = "Auto: on";
            this.auto_interval = setInterval(() => {
                this.step_fn();
            }, this.time_interval);
        } else {
            this.auto_mode = false;
            this.auto_btn.textContent = "Auto: off";
            clearInterval(this.auto_interval);
            this.auto_interval = null;
        }
    }

    setOnLoad(fn) {
        this.load_fn = fn;
    }

    setOnStep(fn) {
        this.step_fn = fn;
    }

    setErrorHandler(fn) {
        this.error_handler = fn;
    }

    handleError(e) {
        this.error_handler(e);
        if (this.auto_mode) {
            console.log("auto stop");
            this.toggleAuto();
        }
    }
}

// control by auto call

// ---- control class end ----
