use std::{cell::RefCell, future::Future, pin::Pin, rc::Rc, sync::Arc};

use deno_core::{
    futures::FutureExt, resolve_import, ModuleLoadId, ModuleSource, ModuleSourceFuture,
    ModuleSpecifier, OpState,
};

use deno_core::{self, error::AnyError, ModuleLoader};

use super::program_state::DenoDevrcProgramState;

#[derive(Debug)]
pub struct DevrcDenoModuleLoader {
    pub state: Arc<DenoDevrcProgramState>,
}

impl DevrcDenoModuleLoader {
    pub fn new(state: Arc<DenoDevrcProgramState>) -> Rc<Self> {
        Rc::new(DevrcDenoModuleLoader { state })
    }
}

impl ModuleLoader for DevrcDenoModuleLoader {
    fn resolve(
        &self,
        _op_state: Rc<RefCell<OpState>>,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<ModuleSpecifier, AnyError> {
        let module_specifier = resolve_import(specifier, referrer)?;

        Ok(module_specifier)
    }

    fn load(
        &self,
        _op_state: Rc<RefCell<OpState>>,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<ModuleSpecifier>,
        _is_dynamic: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let state = self.state.clone();
        let file = state.file_fetcher.cache.get(module_specifier).unwrap();

        let module_specifier = module_specifier.clone();
        async move {
            let module = ModuleSource {
                code: file.source,
                module_url_specified: module_specifier.to_string(),
                module_url_found: file.specifier.to_string(),
            };
            Ok(module)
        }
        .boxed_local()
    }

    fn prepare_load(
        &self,
        _op_state: Rc<RefCell<OpState>>,
        _load_id: ModuleLoadId,
        _module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<String>,
        _is_dyn_import: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), AnyError>>>> {
        async { Ok(()) }.boxed_local()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_name() {}
}
