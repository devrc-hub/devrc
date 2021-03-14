use std::{rc::Rc, sync::Arc};

use crate::errors::DevrcResult;

use deno_core::ModuleSpecifier;

use deno_runtime::{
    ops::worker_host::CreateWebWorkerCb,
    permissions::Permissions,
    web_worker::{WebWorker, WebWorkerOptions},
    worker::{MainWorker, WorkerOptions},
};

use super::{module_loader::DevrcDenoModuleLoader, program_state::DenoDevrcProgramState};
use crate::user_agent::get_user_agent;

use deno_lib::program_state::ProgramState as OriginalProgramState;

use deno_lib::{
    colors, errors::get_error_class_name, fmt_errors::PrettyJsError,
    module_loader::CliModuleLoader, ops, source_maps::apply_source_map, version,
};

pub struct State {}

pub fn create_main_worker(
    state: &Arc<DenoDevrcProgramState>,
    main_module: ModuleSpecifier,
    permissions: Permissions,
) -> DevrcResult<MainWorker> {
    let module_loader = DevrcDenoModuleLoader::new(state.clone());

    let options = WorkerOptions {
        apply_source_maps: false,
        user_agent: get_user_agent(),
        args: vec![],
        debug_flag: false,
        unstable: false,
        ca_data: None,
        seed: None,
        js_error_create_fn: None,
        create_web_worker_cb: Arc::new(|_| unreachable!()),
        attach_inspector: false,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        module_loader,
        runtime_version: "-".to_string(),
        ts_version: "-".to_string(),
        no_color: true,
        get_error_class_fn: Some(&get_error_class_name),
        location: None,
    };

    let mut worker = MainWorker::from_options(main_module, permissions, &options);

    worker.bootstrap(&options);
    Ok(worker)
}

fn create_original_web_worker_callback(
    program_state: Arc<OriginalProgramState>,
) -> Arc<CreateWebWorkerCb> {
    Arc::new(move |args| {
        let global_state_ = program_state.clone();
        let js_error_create_fn = Rc::new(move |core_js_error| {
            let source_mapped_error = apply_source_map(&core_js_error, global_state_.clone());
            PrettyJsError::create(source_mapped_error)
        });

        let attach_inspector =
            program_state.maybe_inspector_server.is_some() || program_state.coverage_dir.is_some();
        let maybe_inspector_server = program_state.maybe_inspector_server.clone();

        let module_loader =
            CliModuleLoader::new_for_worker(program_state.clone(), args.parent_permissions.clone());
        let create_web_worker_cb = create_original_web_worker_callback(program_state.clone());

        let options = WebWorkerOptions {
            args: program_state.flags.argv.clone(),
            apply_source_maps: true,
            debug_flag: program_state
                .flags
                .log_level
                .map_or(false, |l| l == log::Level::Debug),
            unstable: program_state.flags.unstable,
            ca_data: program_state.ca_data.clone(),
            user_agent: get_user_agent(),
            seed: program_state.flags.seed,
            module_loader,
            create_web_worker_cb,
            js_error_create_fn: Some(js_error_create_fn),
            use_deno_namespace: args.use_deno_namespace,
            attach_inspector,
            maybe_inspector_server,
            runtime_version: version::deno(),
            ts_version: version::TYPESCRIPT.to_string(),
            no_color: !colors::use_color(),
            get_error_class_fn: Some(&deno_lib::errors::get_error_class_name),
        };

        let mut worker = WebWorker::from_options(
            args.name,
            args.permissions,
            args.main_module,
            args.worker_id,
            &options,
        );

        // This block registers additional ops and state that
        // are only available in the CLI
        {
            let js_runtime = &mut worker.js_runtime;
            js_runtime
                .op_state()
                .borrow_mut()
                .put::<Arc<OriginalProgramState>>(program_state.clone());
            // Applies source maps - works in conjuction with `js_error_create_fn`
            // above
            ops::errors::init(js_runtime);
            if args.use_deno_namespace {
                ops::runtime_compiler::init(js_runtime);
            }
        }
        worker.bootstrap(&options);

        worker
    })
}

pub fn create_original_main_worker(
    program_state: &Arc<OriginalProgramState>,
    main_module: ModuleSpecifier,
    permissions: Permissions,
) -> MainWorker {
    let module_loader = CliModuleLoader::new(program_state.clone());

    let global_state_ = program_state.clone();

    let js_error_create_fn = Rc::new(move |core_js_error| {
        let source_mapped_error = apply_source_map(&core_js_error, global_state_.clone());
        PrettyJsError::create(source_mapped_error)
    });

    let attach_inspector = program_state.maybe_inspector_server.is_some()
        || program_state.flags.repl
        || program_state.coverage_dir.is_some();
    let maybe_inspector_server = program_state.maybe_inspector_server.clone();
    let should_break_on_first_statement = program_state.flags.inspect_brk.is_some();

    let create_web_worker_cb = create_original_web_worker_callback(program_state.clone());

    let options = WorkerOptions {
        apply_source_maps: true,
        args: program_state.flags.argv.clone(),
        debug_flag: program_state
            .flags
            .log_level
            .map_or(false, |l| l == log::Level::Debug),
        unstable: program_state.flags.unstable,
        ca_data: program_state.ca_data.clone(),
        user_agent: get_user_agent(),
        seed: program_state.flags.seed,
        js_error_create_fn: Some(js_error_create_fn),
        create_web_worker_cb,
        attach_inspector,
        maybe_inspector_server,
        should_break_on_first_statement,
        module_loader,
        runtime_version: version::deno(),
        ts_version: version::TYPESCRIPT.to_string(),
        no_color: !colors::use_color(),
        get_error_class_fn: Some(&deno_lib::errors::get_error_class_name),
        location: program_state.flags.location.clone(),
    };

    let mut worker = MainWorker::from_options(main_module, permissions, &options);

    // This block registers additional ops and state that
    // are only available in the CLI
    {
        let js_runtime = &mut worker.js_runtime;
        js_runtime
            .op_state()
            .borrow_mut()
            .put::<Arc<OriginalProgramState>>(program_state.clone());
        // Applies source maps - works in conjuction with `js_error_create_fn`
        // above
        ops::errors::init(js_runtime);
        ops::runtime_compiler::init(js_runtime);
    }
    worker.bootstrap(&options);

    worker
}
