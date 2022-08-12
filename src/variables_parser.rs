use crate::{
    errors::{DevrcError, DevrcResult},
    variables::VariableKey,
};

const GLOBAL_MODIFIER: &str = "+global";

pub fn parse_key(value: &str) -> DevrcResult<VariableKey> {
    let mut parts = value.splitn(2, |c| c == ' ' || c == '\t');

    let name = parts.next().ok_or(DevrcError::InvalidVariableName)?;

    let mut key = VariableKey {
        original: value.to_string(),
        name: name.to_string(),
        set_global: false,
    };

    if let Some(value) = parts.next() {
        if value == GLOBAL_MODIFIER {
            key.set_global = true;
        } else {
            return Err(DevrcError::InvalidVariableModifier);
        }
    }

    Ok(key)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_variables_key_with_global() {
        let result = parse_key("name +global").unwrap();

        let control = VariableKey {
            original: "name +global".to_string(),
            name: "name".to_string(),
            set_global: true,
        };
        assert_eq!(result, control);
    }

    #[test]
    fn test_parse_variables_key() {
        let result = parse_key("name").unwrap();

        let control = VariableKey {
            original: "name".to_string(),
            name: "name".to_string(),
            set_global: false,
        };
        assert_eq!(result, control);
    }

    #[test]
    fn test_invalid_key() {
        let result = parse_key("name +invalid").unwrap_err();

        //     assert_eq!(result, DevrcError::InvalidVariableModifier);

        //     match devrcfile.find_task("task_3").unwrap() {
        //         TaskKind::ComplexCommand(ComplexCommand {
        //             exec: ExecKind::String(exec),
        //             ..
        //         }) => {
        //             assert_eq!(exec, "echo \"Hello 3\"");
        //         }
        //         _ => {
        //             unreachable!();
        //         }
        //     }

        //     assert_eq!(Config::new(&args).err(), Some("not enough arguments"))

        //          .unwrap_err()
        //     .kind();
        // let expected_error_kind = ErrorKind::PermissionDenied;
        // assert_eq!(actual_error_kind, expected_error_kind);
    }
}
