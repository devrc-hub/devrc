use crate::errors::DevrcResult;
use deno_core::resolve_url_or_path;
use deno_lib::{
    file_fetcher::File, flags::Flags, media_type::MediaType,
    program_state::ProgramState as DProgramState,
};
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

pub fn execute_code(code: &str, permissions_options: PermissionsOptions) -> DevrcResult<()> {
    tokio_rt::run(get_execute_future(
        &code,
        Permissions::from_options(&permissions_options),
    ))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::DevrcError;

    #[test]
    fn test_execute_code() {
        let code = "console.log(\"It's work!\"); console.log(Deno);";
        execute_code(code, PermissionsOptions::default()).unwrap();

        let code = "type WindowStates = \"open\" | \"closed\" | \"minimized\"; let helloWorld = \"Hello World\"; console.log(`Hello ${helloWorld}`)";

        execute_code(code, PermissionsOptions::default()).unwrap();
    }

    #[test]
    fn test_execute_code_fail() {
        let code = "sdf; console.log(\"It's work!\"); console.log(Deno);";
        let res = execute_code(code, PermissionsOptions::default());

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

        let res = execute_code(code, PermissionsOptions::default());

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
