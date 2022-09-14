use std::convert::TryInto;

use tera::{Context, Tera};

use crate::{
    errors::{DevrcError, DevrcResult},
    scope::Scope,
};

pub fn render_string(name: &str, template: &str, scope: &Scope) -> DevrcResult<String> {
    let context: Context = scope.try_into()?;

    let _autoescape = true;

    // TODO: pass tera as input parameter
    let mut tera = Tera::default();

    // if autoescape {
    //     tera.autoescape_on(vec![ONE_OFF_TEMPLATE_NAME]);
    // }

    tera.add_raw_template(name, template)?;
    let result = tera.render(name, &context);
    tera.templates.remove(name);

    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(error.into()), // Err(value) => {
                                         //     // TODO: wrap Tera error
                                         //     // println!("Render template error: {:}", value);
                                         //     Err(DevrcError::RenderError(value))
                                         // }
    }
}

pub fn render_multiple(
    input: indexmap::IndexMap<String, String>,
    scope: &Scope,
) -> DevrcResult<indexmap::IndexMap<String, String>> {
    let mut result = indexmap::IndexMap::new();
    let _autoescape = true;
    let mut context: Context = scope.try_into()?;

    let mut tera = Tera::default();

    for (template_name, template_value) in input.iter() {
        tera.add_raw_template(template_name, template_value)?;
        let rendered_template = tera
            .render(template_name, &context)
            .map_err(DevrcError::RenderError)?;
        tera.templates.remove(template_name);

        context
            .try_insert(template_name, &rendered_template)
            .map_err(DevrcError::RenderError)?;

        result.insert(template_name.to_string(), rendered_template.to_string());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use super::*;
    use tera::{Error as TeraError, ErrorKind as TerraErrorKind};

    use crate::{
        errors::DevrcError,
        scope::Scope,
        variables::{VariableKey, VariableValue},
    };

    #[test]
    fn test_render_string() {
        let mut scope = Scope::default();
        scope.insert_var(
            VariableKey::try_from("name".to_string()).unwrap(),
            VariableValue::new("name", "username")
                .with_render_value(&scope)
                .unwrap(),
        );
        let rendered_template =
            render_string("var_name", "some template string: {{ name }}", &scope).unwrap();

        dbg!(&rendered_template);

        assert_eq!(
            rendered_template,
            "some template string: username".to_owned()
        );
    }

    #[test]
    fn test_render_invalid_template() {
        let rendered_template =
            render_string("var_name", "some template {{ } string", &Scope::default());

        assert!(rendered_template.is_err());

        match rendered_template.err().unwrap() {
            DevrcError::RenderError(TeraError {
                kind: TerraErrorKind::Msg(kind),
                ..
            }) => {
                assert_eq!(kind, "Failed to parse \'var_name\'");
            }
            _ => unreachable!(),
        }
    }
}
