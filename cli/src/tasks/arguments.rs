use indexmap::IndexMap;

use crate::{
    devrcfile::Devrcfile,
    errors::{DevrcError, DevrcResult},
};

use super::{params::ParamValue, TaskKind};

pub type TaskArguments = IndexMap<String, (String, ParamValue)>;

pub fn strip_arg_name(input: String, param_name: &str) -> DevrcResult<String> {
    let prefix = format!("{:}=", param_name);

    if input.starts_with(&prefix) {
        Ok(input
            .strip_prefix(&prefix)
            .ok_or(DevrcError::TaskArgumentsParsingError)?
            .to_string())
    } else {
        Ok(input)
    }
}

// Try to guess is value argument or taskname
pub fn is_argument(devrcfile: &Devrcfile, value: &str) -> bool {
    match devrcfile.find_task(value) {
        Ok(_) => value.contains(' ') || value.contains('='),
        Err(_) => true,
    }
}

pub fn extract_task_args(
    task: &TaskKind,
    parts: &[String],
    devrcfile: &Devrcfile,
) -> DevrcResult<(usize, TaskArguments)> {
    let params = task.get_parameters(parts)?;

    let mut arguments: TaskArguments = indexmap::IndexMap::new();

    let mut taken_arguments_counter = 0;
    for (idx, (key, value)) in params.iter().enumerate() {
        match value {
            ParamValue::Required => {
                if let Some(value) = parts.get(idx) {
                    taken_arguments_counter += 1;
                    arguments.insert(
                        key.to_string(),
                        (
                            strip_arg_name(value.to_string(), key)?,
                            ParamValue::Required,
                        ),
                    )
                } else {
                    return Err(DevrcError::NotEnouthArguments);
                }
            }
            ParamValue::Default(default) => {
                if let Some(value) = parts.get(idx) {
                    if is_argument(devrcfile, value) {
                        taken_arguments_counter += 1;
                        arguments.insert(
                            key.to_string(),
                            (
                                strip_arg_name(value.to_string(), key)?,
                                ParamValue::Default(default.clone()),
                            ),
                        )
                    } else {
                        arguments.insert(
                            key.to_string(),
                            (default.clone(), ParamValue::Default(default.clone())),
                        )
                    }
                } else {
                    arguments.insert(
                        key.to_string(),
                        (default.clone(), ParamValue::Default(default.clone())),
                    )
                }
            }
        };
    }

    Ok((taken_arguments_counter, arguments))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_name() {}
}
