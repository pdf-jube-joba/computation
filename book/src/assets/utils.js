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

// ---- control class ----
// class provide controls of the machine
// - load
// - run (one step)

// controls by user
// which calls
// (user can auto_step and auto_stop)
export class UserControls {
    constructor(step_id, load_id, auto_id) {
        this.step_btn = document.getElementById(step_id);
        this.load_btn = document.getElementById(load_id);
        this.auto_btn = document.getElementById(auto_id);

        this.step_fn = () => {};
        this.load_fn = () => {};
        // auto_step_fn は将来用意（未定義でOK）

        this.step_btn?.addEventListener("click", () => this.step_fn());
        this.load_btn?.addEventListener("click", () => this.load_fn());
    }

    onLoad(fn) {
        this.load_fn = fn;
    }

    onStep(fn) {
        this.step_fn = fn;
    }
}


// control by auto call

// ---- control class end ----
