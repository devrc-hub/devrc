use crate::{errors::DevrcResult, interpreter::DenoPermission};
use deno_3p_lib::{
    file_fetcher::File, flags::Flags, media_type::MediaType,
    program_state::ProgramState as DProgramState,
};
use deno_core::resolve_url_or_path;
use deno_runtime::permissions::{Permissions, PermissionsOptions};
use workers::create_original_main_worker;

pub mod module_loader;
pub mod program_state;
pub mod tokio_rt;
pub mod workers;

async fn get_execute_future(code: &str, permissions: Permissions) -> DevrcResult<()> {
    let program_state = DProgramState::build(Flags::default()).await?;
    let main_module = resolve_url_or_path("./$deno$devrc_task.ts").unwrap();

    // Create a dummy source file.
    let source_file = File {
        local: main_module.clone().to_file_path().unwrap(),
        maybe_types: None,
        media_type: MediaType::TypeScript,
        source: code.to_owned(),
        specifier: main_module.clone(),
    };

    program_state.file_fetcher.insert_cached(source_file);

    let mut worker =
        create_original_main_worker(&program_state.clone(), main_module.clone(), permissions);

    worker.execute_module(&main_module).await?;

    worker.execute("window.dispatchEvent(new Event('load'))")?;
    worker.run_event_loop().await?;
    worker.execute("window.dispatchEvent(new Event('unload'))")?;
    Ok(())
}

// async fn get_execute_future(code: &str) -> DevrcResult<()> {
//     let permissions = Permissions::allow_all();
//     let state = Arc::new(ProgramState::default());
//     let program_state = DProgramState::build(Flags::default());
//     let main_module = resolve_url_or_path("./$deno$devrc_task.ts").unwrap();

//     // Create a dummy source file.
//     let source_file = File {
//         local: main_module.clone().to_file_path().unwrap(),
//         maybe_types: None,
//         media_type: MediaType::TypeScript,
//         source: code.to_owned(),
//         specifier: main_module.clone(),
//     };

//     state.file_fetcher.insert_cached(source_file);

//     let mut worker = create_main_worker(&state.clone(), main_module.clone(), permissions)?;

//     let _execute_res = worker.execute_module(&main_module).await;
//     // dbg!(&execute_res);

//     worker.execute("window.dispatchEvent(new Event('load'))")?;
//     worker.run_event_loop().await?;
//     worker.execute("window.dispatchEvent(new Event('unload'))")?;
//     Ok(())
// }

pub fn get_deno_permissions(permissions: &Option<Vec<DenoPermission>>) -> PermissionsOptions {
    let mut permissions_options = PermissionsOptions::default();

    if let Some(permissions_list) = &permissions {
        for permission in permissions_list {
            match permission {
                DenoPermission::DisableAll => return PermissionsOptions::default(),
                DenoPermission::AllowAll => {
                    permissions_options.allow_env = true;
                    permissions_options.allow_hrtime = true;
                    permissions_options.allow_net = Some(Vec::new());
                    permissions_options.allow_plugin = true;
                    permissions_options.allow_read = Some(Vec::new());
                    permissions_options.allow_run = true;
                    permissions_options.allow_write = Some(Vec::new());
                    return permissions_options;
                }
                DenoPermission::AllowEnv => {
                    permissions_options.allow_env = true;
                }
                DenoPermission::AllowHrtime => {
                    permissions_options.allow_hrtime = true;
                }
                DenoPermission::AllowPlugin => {
                    permissions_options.allow_plugin = true;
                }
                DenoPermission::AllowRun => {
                    permissions_options.allow_run = true;
                }
                DenoPermission::AllowWriteAll => {
                    permissions_options.allow_write = Some(Vec::new());
                }
                DenoPermission::AllowReadAll => {
                    permissions_options.allow_read = Some(Vec::new());
                }
                DenoPermission::AllowNetAll => {
                    permissions_options.allow_read = Some(Vec::new());
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
            }
        }
    }

    permissions_options
}

pub fn execute_deno_code(code: &str, permissions: &Option<Vec<DenoPermission>>) -> DevrcResult<()> {
    tokio_rt::run(get_execute_future(
        code,
        Permissions::from_options(&get_deno_permissions(permissions)),
    ))?
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
