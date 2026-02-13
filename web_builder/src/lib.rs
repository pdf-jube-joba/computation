// macro for generate MACHINEs

#[macro_export]
// Define a macro for creating a web model interface
// which holds a MACHINE instance and exposes functions to interact with it via WebAssembly.
// user should define MachineType as the target MACHINE type before invoking this macro.
// arguments: type of the MACHINE
macro_rules! web_model {
    ($x:tt) => {
        // binary としてコンパイルするので、あまり意味がないが
        // エラーが出るから、一応 main 関数を定義しておく
        fn main() {}

        type MachineType = $x;

        use wasm_bindgen::prelude::*;

        pub trait WebView {
            fn step(&mut self, rinput: &str) -> Result<Option<String>, String>;
            fn current(&self) -> Result<JsValue, JsValue>;
        }

        impl<T> WebView for T
        where
            T: utils::Machine,
        {
            fn step(&mut self, rinput: &str) -> Result<Option<String>, String> {
                let parsed = <Self as utils::Machine>::parse_rinput(rinput)?;
                let output = <Self as utils::Machine>::step(self, parsed)?;
                match output {
                    Some(o) => {
                        // let s = o.print();
                        let s = <<T as utils::Machine>::Output as utils::TextCodec>::print(&o);
                        Ok(Some(s))
                    },
                    None => Ok(None),
                }
            }

            fn current(&self) -> Result<JsValue, JsValue> {
                serde_wasm_bindgen::to_value(&<Self as utils::Machine>::current(self))
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }

        thread_local! {
            static MACHINE: std::cell::RefCell<Option<Box<dyn WebView>>> = std::cell::RefCell::new(None);
        }

        #[wasm_bindgen]
        pub fn step_machine(rinput: &str) -> Result<Option<String>, JsValue> {
            MACHINE.with(|machine| {
                let mut machine = machine.borrow_mut();
                let m = machine
                    .as_mut()
                    .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
                let result = m.step(rinput).map_err(|e| JsValue::from_str(&e))?;
                Ok(result)
            })
        }

        #[wasm_bindgen]
        pub fn current_machine() -> Result<JsValue, JsValue> {
            MACHINE.with(|machine| {
                let machine = machine.borrow();
                let m = machine
                    .as_ref()
                    .ok_or_else(|| JsValue::from_str("Machine not initialized"))?;
                m.current()
            })
        }

        #[allow(dead_code)]
        fn create_machine<T: utils::Machine + 'static>(
            code: &str,
            ainput: &str,
        ) -> Result<(), JsValue> {
            MACHINE.with(|machine| {
                let mut machine = machine.borrow_mut();
                *machine = None;
            });
            let code = T::parse_code(code).map_err(|e| JsValue::from_str(&e))?;
            let ainput = T::parse_ainput(ainput).map_err(|e| JsValue::from_str(&e))?;
            let machine = T::make(code, ainput).map_err(|e| JsValue::from_str(&e))?;
            let boxed: Box<dyn WebView> = Box::new(machine);
            MACHINE.with(|machine| {
                let mut machine = machine.borrow_mut();
                *machine = Some(boxed);
                Ok(())
            })
        }

        #[wasm_bindgen]
        pub fn create(input: &str, ainput: &str) -> Result<(), JsValue> {
            create_machine::<MachineType>(input, ainput)
        }
    };
}
