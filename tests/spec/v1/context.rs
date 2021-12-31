#![allow(dead_code)] // TODO: remove

use super::{TestError, TestProfile};
use anyhow::Result;
use std::collections::HashMap;
use wasmi::{
    nan_preserving_float::{F32, F64},
    v1::{
        Engine,
        Func,
        Global,
        Instance,
        Linker,
        Memory,
        MemoryType,
        Module,
        Mutability,
        Store,
        Table,
        TableType,
    },
    RuntimeValue,
};
use wast::Id;

/// The context of a single Wasm test spec suite run.
#[derive(Debug)]
pub struct TestContext {
    /// The `wasmi` engine used for executing functions used during the test.
    engine: Engine,
    /// The linker for linking together Wasm test modules.
    linker: Linker<()>,
    /// The store to hold all runtime data during the test.
    store: Store<()>,
    /// The list of all encountered Wasm modules belonging to the test.
    modules: Vec<Module>,
    /// The list of all instantiated modules.
    instances: HashMap<String, Instance>,
    /// The last touched module instance.
    last_instance: Option<Instance>,
    /// Profiling during the Wasm spec test run.
    profile: TestProfile,
}

impl Default for TestContext {
    fn default() -> Self {
        let engine = Engine::default();
        let mut linker = Linker::default();
        let mut store = Store::new(&engine, ());
        let default_memory = Memory::new(&mut store, MemoryType::new(1, Some(2))).unwrap();
        let default_table = Table::new(&mut store, TableType::new(10, Some(20)));
        let global_i32 = Global::new(&mut store, RuntimeValue::I32(666), Mutability::Const);
        let print_i32 = Func::wrap(&mut store, |value: i32| {
            println!("print: {}", value);
        });
        let print_f32 = Func::wrap(&mut store, |value: F32| {
            println!("print: {:?}", value);
        });
        let print_f64 = Func::wrap(&mut store, |value: F64| {
            println!("print: {:?}", value);
        });
        let print_i32_f32 = Func::wrap(&mut store, |v0: i32, v1: F32| {
            println!("print: {:?} {:?}", v0, v1);
        });
        let print_f64_f64 = Func::wrap(&mut store, |v0: F64, v1: F64| {
            println!("print: {:?} {:?}", v0, v1);
        });
        linker.define("spectest", "memory", default_memory).unwrap();
        linker.define("spectest", "table", default_table).unwrap();
        linker.define("spectest", "global_i32", global_i32).unwrap();
        linker.define("spectest", "print_i32", print_i32).unwrap();
        linker.define("spectest", "print_f32", print_f32).unwrap();
        linker.define("spectest", "print_f64", print_f64).unwrap();
        linker
            .define("spectest", "print_i32_f32", print_i32_f32)
            .unwrap();
        linker
            .define("spectest", "print_f64_f64", print_f64_f64)
            .unwrap();
        TestContext {
            engine,
            linker,
            store,
            modules: Vec::new(),
            instances: HashMap::new(),
            last_instance: None,
            profile: TestProfile::default(),
        }
    }
}

impl TestContext {
    /// Returns the [`Engine`] of the [`TestContext`].
    fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Returns an exclusive reference to the test profile.
    pub fn profile(&mut self) -> &mut TestProfile {
        &mut self.profile
    }

    /// Compiles the Wasm module and stores it into the [`TestContext`].
    ///
    /// # Errors
    ///
    /// If creating the [`Module`] fails.
    pub fn compile_and_instantiate(
        &mut self,
        id: Option<Id>,
        wasm: impl AsRef<[u8]>,
    ) -> Result<Instance> {
        let module = Module::new(self.engine(), wasm.as_ref())?;
        let instance_pre = self.linker.instantiate(&mut self.store, &module)?;
        let instance = instance_pre.ensure_no_start_fn(&mut self.store)?;
        self.modules.push(module);
        if let Some(name) = id.map(|id| id.name()) {
            self.instances.insert(name.to_string(), instance);
        }
        self.last_instance = Some(instance);
        Ok(instance)
    }

    /// Loads the Wasm module instance with the given name.
    ///
    /// # Errors
    ///
    /// If there is no registered module instance with the given name.
    pub fn instance_by_name(&self, name: &str) -> Result<Instance, TestError> {
        self.instances
            .get(name)
            .copied()
            .ok_or_else(|| TestError::InstanceNotRegistered {
                name: name.to_owned(),
            })
    }

    /// Loads the Wasm module instance with the given name or the last instantiated one.
    ///
    /// # Errors
    ///
    /// If there have been no Wasm module instances registered so far.
    pub fn instance_by_name_or_last(&self, name: Option<&str>) -> Result<Instance, TestError> {
        name.map(|name| self.instance_by_name(name))
            .unwrap_or_else(|| {
                self.last_instance
                    .ok_or_else(|| TestError::NoModuleInstancesFound)
            })
    }
}