
use tera::{Context, Tera};

use crate::{
    errors::{DevrcResult},
    scope::Scope,
};

pub fn render_string(name: &str, template: &str, scope: &Scope) -> DevrcResult<String> {
    let context: Context = scope.into();

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

#[cfg(test)]
mod tests {
    use super::*;
    use tera::{Error as TeraError, ErrorKind as TerraErrorKind};

    use crate::{
        errors::{DevrcError},
        scope::Scope,
    };


    #[test]
    fn test_render_string() {
        let mut scope = Scope::default();
        scope.insert_var("name", "username");
        let rendered_template =
            render_string("var_name", "some template string: {{ name }}", &scope);

        assert_eq!(
            rendered_template.unwrap(),
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
            _ => assert!(false),
        }
    }
}
