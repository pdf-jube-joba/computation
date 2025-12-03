use std::cell::RefCell;
use utils::WebView;
use wasm_bindgen::prelude::*;

thread_local! {
    static MACHINE: RefCell<Vec<Box<dyn WebView>>> = RefCell::new(vec![]);
}

#[wasm_bindgen]
pub fn step_machine(index: usize, input: &str) -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        let m = machine
            .get_mut(index)
            .ok_or_else(|| JsValue::from_str("Invalid machine index"))?;
        let result = m.step(input).map_err(|e| JsValue::from_str(&e))?;
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

#[wasm_bindgen]
pub fn current_machine(index: usize) -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let machine = machine.borrow();
        let m = machine
            .get(index)
            .ok_or_else(|| JsValue::from_str("Invalid machine index"))?;
        let result = m.current();
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

#[wasm_bindgen]
pub fn output_machine(index: usize) -> Result<JsValue, JsValue> {
    MACHINE.with(|machine| {
        let machine = machine.borrow();
        let m = machine
            .get(index)
            .ok_or_else(|| JsValue::from_str("Invalid machine index"))?;
        let result = m
            .output()
            .ok_or_else(|| JsValue::from_str("Machine not terminated"))?;
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

#[wasm_bindgen]
pub fn default_machine() -> usize {
    MACHINE.with(|machine| {
        let mut machine = machine.borrow_mut();
        let m: Box<dyn WebView> = Box::new(example::Counter { count: 0 });
        machine.push(m);
        machine.len() - 1
    })
}

mod example {
    use utils::WebView;

    pub struct Counter {
        pub count: usize,
    }
    impl WebView for Counter {
        fn step(&mut self, input: &str) -> Result<serde_json::Value, String> {
            match input {
                "increment" => {
                    self.count += 1;
                    Ok(serde_json::json!({"status": "ok", "count": self.count}))
                }
                "decrement" => {
                    if self.count == 0 {
                        Err("Count cannot be negative".to_string())
                    } else {
                        self.count -= 1;
                        Ok(serde_json::json!({"status": "ok", "count": self.count}))
                    }
                }
                _ => Err("Invalid input".to_string()),
            }
        }

        fn current(&self) -> serde_json::Value {
            serde_json::json!({"count": self.count})
        }

        fn output(&self) -> Option<serde_json::Value> {
            None
        }
    }
}
