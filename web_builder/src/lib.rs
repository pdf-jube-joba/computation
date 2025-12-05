use std::cell::RefCell;
use utils::WebView;
use wasm_bindgen::prelude::*;

thread_local! {
    static MACHINE: RefCell<Option<Box<dyn WebView>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn step_machine(input: &str) -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        let m = machine
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
        let result = m.step(input).map_err(|e| JsValue::from_str(&e))?;
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

#[wasm_bindgen]
pub fn current_machine() -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let machine = machine.borrow();
        let m = machine
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
        let result = m.current();
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

#[wasm_bindgen]
pub fn create(input: &str) {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        let initial_count = input.trim().parse::<usize>().unwrap_or(0);
        let m: Box<dyn WebView> = Box::new(example::Counter { count: initial_count });
        *machine = Some(m);
    })
}

mod example {
    use utils::WebView;

    pub struct Counter {
        pub count: usize,
    }
    impl WebView for Counter {
        fn step(&mut self, input: &str) -> Result<Option<serde_json::Value>, String> {
            match input {
                "increment" => {
                    self.count += 1;
                    Ok(None)
                }
                "decrement" => {
                    if self.count == 0 {
                        Err("Count cannot be negative".to_string())
                    } else {
                        self.count -= 1;
                        Ok(None)
                    }
                }
                _ => Err("Invalid input".to_string()),
            }
        }

        fn current(&self) -> serde_json::Value {
            serde_json::json!({"count": self.count})
        }
    }
}
