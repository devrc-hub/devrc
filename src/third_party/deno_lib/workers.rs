// #![allow(unused_imports)]


use crate::fmt_errors::PrettyJsError;
use crate::module_loader::CliModuleLoader;
use crate::program_state::ProgramState;
use crate::source_maps::apply_source_map;
use crate::version;
use crate::colors;
use crate::ops;


use deno_core::ModuleSpecifier;
use deno_runtime::{ops::worker_host::CreateWebWorkerCb, permissions::Permissions, web_worker::{WebWorker, WebWorkerOptions}, worker::{MainWorker, WorkerOptions}};


use std::rc::Rc;
use std::sync::Arc;


fn create_web_worker_callback(
  program_state: Arc<ProgramState>,
) -> Arc<CreateWebWorkerCb> {
  Arc::new(move |args| {
    let global_state_ = program_state.clone();
    let js_error_create_fn = Rc::new(move |core_js_error| {
      let source_mapped_error =
        apply_source_map(&core_js_error, global_state_.clone());
      PrettyJsError::create(source_mapped_error)
    });

    let attach_inspector = program_state.maybe_inspector_server.is_some()
      || program_state.coverage_dir.is_some();
    let maybe_inspector_server = program_state.maybe_inspector_server.clone();

    let module_loader = CliModuleLoader::new_for_worker(
      program_state.clone(),
      args.parent_permissions.clone(),
    );
    let create_web_worker_cb =
      create_web_worker_callback(program_state.clone());

    let options = WebWorkerOptions {
      args: program_state.flags.argv.clone(),
      apply_source_maps: true,
      debug_flag: program_state
        .flags
        .log_level
        .map_or(false, |l| l == log::Level::Debug),
      unstable: program_state.flags.unstable,
      ca_data: program_state.ca_data.clone(),
      user_agent: version::get_user_agent(),
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
      get_error_class_fn: Some(&crate::errors::get_error_class_name),
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
        .put::<Arc<ProgramState>>(program_state.clone());
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

pub fn create_main_worker(
  program_state: &Arc<ProgramState>,
  main_module: ModuleSpecifier,
  permissions: Permissions,
) -> MainWorker {
  let module_loader = CliModuleLoader::new(program_state.clone());

  let global_state_ = program_state.clone();

  let js_error_create_fn = Rc::new(move |core_js_error| {
    let source_mapped_error =
      apply_source_map(&core_js_error, global_state_.clone());
    PrettyJsError::create(source_mapped_error)
  });

  let attach_inspector = program_state.maybe_inspector_server.is_some()
    || program_state.flags.repl
    || program_state.coverage_dir.is_some();
  let maybe_inspector_server = program_state.maybe_inspector_server.clone();
  let should_break_on_first_statement =
    program_state.flags.inspect_brk.is_some();

  let create_web_worker_cb = create_web_worker_callback(program_state.clone());

  let options = WorkerOptions {
    apply_source_maps: true,
    args: program_state.flags.argv.clone(),
    debug_flag: program_state
      .flags
      .log_level
      .map_or(false, |l| l == log::Level::Debug),
    unstable: program_state.flags.unstable,
    ca_data: program_state.ca_data.clone(),
    user_agent: version::get_user_agent(),
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
    get_error_class_fn: Some(&crate::errors::get_error_class_name),
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
      .put::<Arc<ProgramState>>(program_state.clone());
    // Applies source maps - works in conjuction with `js_error_create_fn`
    // above
    ops::errors::init(js_runtime);
    ops::runtime_compiler::init(js_runtime);
  }
  worker.bootstrap(&options);

  worker
}
