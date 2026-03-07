use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

mod model_bindings {
    wasmtime::component::bindgen!({
        path: "../utils/wit",
        world: "model",
    });
}

mod compiler_bindings {
    wasmtime::component::bindgen!({
        path: "../utils/wit",
        world: "compiler",
    });
}

fn resolve_component_path(name: &str) -> PathBuf {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli has workspace parent");
    workspace
        .join("wasm_bundle")
        .join(format!("{name}.component.wasm"))
}

fn new_engine() -> Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    Engine::new(&config).context("failed to initialize wasmtime engine")
}

fn expect_ok<T>(label: &str, value: std::result::Result<T, String>) -> Result<T> {
    value.map_err(|e| anyhow::anyhow!("{label}: {e}"))
}

pub struct ModelHost {
    store: Store<()>,
    instance: model_bindings::Model,
}

impl ModelHost {
    pub fn load(name: &str) -> Result<Self> {
        let component_path = resolve_component_path(name);
        if !component_path.exists() {
            anyhow::bail!("component not found: {}", component_path.display());
        }

        let engine = new_engine()?;
        let component = Component::from_file(&engine, &component_path)
            .with_context(|| format!("failed to load component: {}", component_path.display()))?;
        let linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        let instance = model_bindings::Model::instantiate(&mut store, &component, &linker)
            .context("instantiate as model failed")?;
        Ok(Self { store, instance })
    }

    pub fn create(&mut self, code: &str, ainput: &str) -> Result<()> {
        let out = self
            .instance
            .call_make(&mut self.store, code, ainput)
            .context("model make failed")?;
        expect_ok("model make", out)
    }

    pub fn step(&mut self, rinput: &str) -> Result<String> {
        let out = self
            .instance
            .call_step(&mut self.store, rinput)
            .context("model step failed")?;
        expect_ok("model step", out)
    }

    pub fn checkpoint(&mut self) -> Result<String> {
        let out = self
            .instance
            .call_snapshot(&mut self.store)
            .context("model snapshot failed")?;
        expect_ok("model snapshot", out)
    }

    pub fn restore(&mut self, snapshot: &str) -> Result<()> {
        let out = self
            .instance
            .call_restore(&mut self.store, snapshot)
            .context("model restore failed")?;
        expect_ok("model restore", out)
    }
}

pub struct CompilerHost {
    store: Store<()>,
    instance: compiler_bindings::Compiler,
}

impl CompilerHost {
    pub fn load(name: &str) -> Result<Self> {
        let component_path = resolve_component_path(name);
        if !component_path.exists() {
            anyhow::bail!("component not found: {}", component_path.display());
        }

        let engine = new_engine()?;
        let component = Component::from_file(&engine, &component_path)
            .with_context(|| format!("failed to load component: {}", component_path.display()))?;
        let linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        let instance = compiler_bindings::Compiler::instantiate(&mut store, &component, &linker)
            .context("instantiate as compiler failed")?;
        Ok(Self { store, instance })
    }

    pub fn compile_code(&mut self, code: &str) -> Result<String> {
        let out = self
            .instance
            .call_compile_code(&mut self.store, code)
            .context("compiler compile-code failed")?;
        expect_ok("compiler compile-code", out)
    }

    pub fn compile_ainput(&mut self, ainput: &str) -> Result<String> {
        let out = self
            .instance
            .call_encode_ainput(&mut self.store, ainput)
            .context("compiler encode-ainput failed")?;
        expect_ok("compiler encode-ainput", out)
    }

    pub fn compile_rinput(&mut self, rinput: &str) -> Result<String> {
        let out = self
            .instance
            .call_encode_rinput(&mut self.store, rinput)
            .context("compiler encode-rinput failed")?;
        expect_ok("compiler encode-rinput", out)
    }

    pub fn decode_routput(&mut self, output: &str) -> Result<String> {
        let out = self
            .instance
            .call_decode_routput(&mut self.store, output)
            .context("compiler decode-routput failed")?;
        expect_ok("compiler decode-routput", out)
    }

    pub fn decode_foutput(&mut self, output: &str) -> Result<String> {
        let out = self
            .instance
            .call_decode_foutput(&mut self.store, output)
            .context("compiler decode-foutput failed")?;
        expect_ok("compiler decode-foutput", out)
    }
}
