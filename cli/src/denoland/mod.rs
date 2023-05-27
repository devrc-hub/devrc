use std::path::PathBuf;

use crate::{errors::DevrcResult, interpreter::DenoPermission};
use deno_3p_lib::{
    args::flags::Flags, file_fetcher::File, proc_state::ProcState, worker::create_main_worker,
};
use deno_ast::MediaType;

use deno_core::resolve_url_or_path;
use deno_runtime::{
    permissions::{Permissions, PermissionsOptions},
    tokio_util::run_local,
};
// use workers::create_original_main_worker;

// pub mod module_loader;
pub mod program_state;
pub mod tokio_rt;

async fn get_execute_future(code: &str, permissions: Permissions) -> DevrcResult<i32> {
    let ps = ProcState::build(Flags::default()).await?;

    let main_module = resolve_url_or_path("./$deno$devrc_task.ts").unwrap();

    // Create a dummy source file.
    let source_file = File {
        local: main_module.clone().to_file_path().unwrap(),
        maybe_types: None,
        media_type: MediaType::TypeScript,
        source: code.into(),
        specifier: main_module.clone(),
        maybe_headers: None,
    };

    ps.file_fetcher.insert_cached(source_file);

    let mut worker = create_main_worker(
        &ps,
        main_module.clone(),
        permissions,
        vec![],
        Default::default(),
    )
    .await?;

    let exit_code = worker.run().await?;
    Ok(exit_code)
}

pub fn get_deno_permissions(permissions: &Option<Vec<DenoPermission>>) -> PermissionsOptions {
    let mut permissions_options = PermissionsOptions::default();

    if let Some(permissions_list) = &permissions {
        for permission in permissions_list {
            match permission {
                DenoPermission::DisableAll => return PermissionsOptions::default(),
                DenoPermission::AllowAll => {
                    permissions_options.allow_env = Some(Vec::new());
                    permissions_options.allow_hrtime = true;
                    permissions_options.allow_net = Some(Vec::new());
                    permissions_options.allow_ffi = Some(Vec::new());
                    permissions_options.allow_read = Some(vec![PathBuf::from("/")]);
                    permissions_options.allow_run = Some(Vec::new());
                    permissions_options.allow_write = Some(vec![PathBuf::from("/")]);

                    return permissions_options;
                }
                DenoPermission::AllowEnv(values) => {
                    permissions_options.allow_env = Some(values.iter().map(|x| x.into()).collect());
                }
                DenoPermission::AllowHrtime => {
                    permissions_options.allow_hrtime = true;
                }
                DenoPermission::AllowFFI(values) => {
                    permissions_options.allow_ffi = Some(values.iter().map(|x| x.into()).collect());
                }
                DenoPermission::AllowRun(values) => {
                    permissions_options.allow_run = Some(values.iter().map(|x| x.into()).collect());
                }
                DenoPermission::AllowNet(values) => {
                    permissions_options.allow_net = Some(values.iter().map(|x| x.into()).collect());
                }
                DenoPermission::AllowRead(values) => {
                    permissions_options.allow_read =
                        Some(values.iter().map(|x| x.into()).collect());
                }
                DenoPermission::AllowWrite(values) => {
                    permissions_options.allow_write =
                        Some(values.iter().map(|x| x.into()).collect());
                }
                DenoPermission::AllowWriteAll => {
                    permissions_options.allow_write = Some(Vec::new());
                }
                DenoPermission::AllowReadAll => {
                    permissions_options.allow_read = Some(Vec::new());
                }
                DenoPermission::AllowNetAll => {
                    permissions_options.allow_net = Some(Vec::new());
                }
                DenoPermission::AllowRunAll => {
                    permissions_options.allow_run = Some(Vec::new());
                }
                DenoPermission::AllowFFIAll => {
                    permissions_options.allow_ffi = Some(Vec::new());
                }
            }
        }
    }

    permissions_options
}

pub fn execute_deno_code(
    code: &str,
    permissions: &Option<Vec<DenoPermission>>,
) -> DevrcResult<i32> {
    run_local(get_execute_future(
        code,
        Permissions::from_options(&get_deno_permissions(permissions))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::DevrcError;

    #[test]
    fn test_execute_deno_code() {
        let code = "console.log(\"It's work!\"); console.log(Deno);";
        let permissions = Some(vec![DenoPermission::DisableAll]);
        execute_deno_code(code, &permissions).unwrap();

        let code = "type WindowStates = \"open\" | \"closed\" | \"minimized\"; let helloWorld = \"Hello World\"; console.log(`Hello ${helloWorld}`)";

        execute_deno_code(code, &permissions).unwrap();
    }

    #[test]
    fn test_execute_deno_code_fail() {
        let code = "sdf; console.log(\"It's work!\"); console.log(Deno);";
        let permissions = Some(vec![DenoPermission::DisableAll]);

        let res = execute_deno_code(code, &permissions);

        match res {
            Err(DevrcError::DenoRuntimeError(_err)) => {}
            Err(_) => {
                unreachable!()
            }
            Ok(_) => {
                unreachable!()
            }
        };

        let code = r#"const res = await fetch("https://httpbin.org/get");"#;

        let res = execute_deno_code(code, &permissions);

        match res {
            Err(DevrcError::DenoRuntimeError(_err)) => {}
            Err(_) => {
                unreachable!()
            }
            Ok(_) => {
                unreachable!()
            }
        };
    }
}
