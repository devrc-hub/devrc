use crate::errors::{DevrcError, DevrcResult};

use super::params::{ParamValue, Params};

#[derive(Debug, Clone, Copy)]
enum ParamsParseState {
    Start,
    End,
    Parameter,
    ParsingParameterValue,
    ParsingParameterEnd,
    EqualSign,
    Separator,
}

pub fn parse_params_string(value: &str) -> DevrcResult<Params> {
    let mut chars = value.chars().enumerate().peekable();

    let mut state: ParamsParseState = ParamsParseState::Start;
    let mut params = Params::default();

    let mut current_parameter: String = String::new();
    let mut current_value: String = String::new();

    let mut positional_done = false;

    loop {
        state = match state {
            ParamsParseState::Start
            | ParamsParseState::Separator
            | ParamsParseState::ParsingParameterEnd => match chars.peek() {
                Some((_idx, value)) if value.is_whitespace() => {
                    chars.next();
                    ParamsParseState::Start
                }
                Some((_idx, _value)) => ParamsParseState::Parameter,
                None => ParamsParseState::End,
            },
            ParamsParseState::Parameter => {
                match chars.next() {
                    Some((_, '=')) => {
                        positional_done = true;
                        ParamsParseState::EqualSign
                    }
                    Some((_, x)) if x.is_whitespace() => {
                        if positional_done {
                            return Err(DevrcError::TaskArgumentsParsingError);
                        }
                        params.add(&current_parameter, &ParamValue::Required)?;
                        current_parameter.clear();
                        ParamsParseState::Separator
                    }
                    Some((_, x)) if (x == '+' || x == '*' || x == '_') => {
                        current_parameter.push(x);
                        ParamsParseState::Parameter
                    }
                    Some((_, x)) if !x.is_alphanumeric() => {
                        return Err(DevrcError::TaskArgumentsParsingError)
                    }
                    // Collect current char to parameter name
                    Some((_, x)) => {
                        current_parameter.push(x);
                        ParamsParseState::Parameter
                    }
                    None => {
                        params.add(&current_parameter, &ParamValue::Required)?;
                        ParamsParseState::End
                    }
                }
            }
            ParamsParseState::EqualSign => match chars.peek() {
                Some((_idx, value)) if value.is_whitespace() => {
                    chars.next();
                    ParamsParseState::EqualSign
                }
                Some((_idx, '"')) => {
                    chars.next();
                    ParamsParseState::ParsingParameterValue
                }
                Some((_idx, _)) => return Err(DevrcError::TaskArgumentsParsingError),
                None => return Err(DevrcError::TaskArgumentsParsingError),
            },
            ParamsParseState::ParsingParameterValue => {
                match chars.next() {
                    Some((_idx, '\\')) => {
                        match chars.peek() {
                            Some((_idx, '"')) => {
                                chars.next();
                                current_value.push('"');
                            }
                            Some((_, '\'')) => {
                                chars.next();
                                current_value.push('\'');
                            }
                            Some((_idx, '\\')) => {
                                chars.next();
                                current_value.push('\\');
                            }
                            Some((_idx, value)) => {
                                current_value.push(*value);
                            }
                            None => {}
                        };
                        ParamsParseState::ParsingParameterValue
                    }
                    Some((_idx, '"')) => {
                        params.add(
                            &current_parameter,
                            &ParamValue::Default(current_value.clone()),
                        )?;
                        current_value.clear();
                        current_parameter.clear();
                        ParamsParseState::ParsingParameterEnd
                    }
                    Some((_idx, value)) => {
                        current_value.push(value);
                        ParamsParseState::ParsingParameterValue
                    }

                    // Close quote not found
                    None => return Err(DevrcError::TaskArgumentsParsingError),
                }
            }
            ParamsParseState::End => break,
        };
    }

    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let input = r#"param1 param2=" value2" param3=  "value3  "  "#;

        let params: Params = parse_params_string(input).unwrap();

        let mut control_params = Params::default();
        control_params.add("param1", &ParamValue::Required).unwrap();
        control_params
            .add("param2", &ParamValue::Default(" value2".to_string()))
            .unwrap();
        control_params
            .add("param3", &ParamValue::Default("value3  ".to_string()))
            .unwrap();

        assert_eq!(params, control_params);

        let input = r#"param1 param2="value2" param3 param4"#;

        match parse_params_string(input).unwrap_err() {
            DevrcError::TaskArgumentsParsingError => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_default_values_with_quotes_inside() {
        let input = r#"param1 param2="string with \" quotes \' inside and simple \\ " "#;

        let params = parse_params_string(input).unwrap();

        let mut control_params = Params::default();
        control_params.add("param1", &ParamValue::Required).unwrap();
        control_params
            .add(
                "param2",
                &ParamValue::Default("string with \" quotes \' inside and simple \\ ".to_string()),
            )
            .unwrap();

        assert_eq!(params, control_params);
    }

    #[test]
    fn test_required() {
        let input = r#"  param1   param2  param_3  +param_4 *param_5"#;

        let params: Params = parse_params_string(input).unwrap();

        let mut control_params = Params::default();
        control_params.add("param1", &ParamValue::Required).unwrap();
        control_params.add("param2", &ParamValue::Required).unwrap();
        control_params
            .add("param_3", &ParamValue::Required)
            .unwrap();
        control_params
            .add("+param_4", &ParamValue::Required)
            .unwrap();
        control_params
            .add("*param_5", &ParamValue::Required)
            .unwrap();

        assert_eq!(params, control_params);
    }

    #[test]
    fn test_invalid_parameter_name() {
        let input = r#"  pa±ram1   param2  "#;

        match parse_params_string(input).unwrap_err() {
            DevrcError::TaskArgumentsParsingError => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_invalid_default() {
        let input = r#"param1=  "#;

        match parse_params_string(input).unwrap_err() {
            DevrcError::TaskArgumentsParsingError => {}
            _ => unreachable!(),
        }

        let input = r#"param1="sdfsdfsdf  "#;

        match parse_params_string(input).unwrap_err() {
            DevrcError::TaskArgumentsParsingError => {}
            _ => unreachable!(),
        }
    }
}
