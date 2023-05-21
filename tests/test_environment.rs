use devrc::{
    env_file::{EnvFilesInclude, LocalFileImport, UrlImport},
    environment::RawEnvironment,
};
use std::fmt::Debug;

use serde::Deserialize;

#[test]
fn test_environment_des_variant_1() {
    let content = r#"ENV_VAR_1: env_var_1_value
"#;

    let env: RawEnvironment<String> =
        serde_yaml::from_str::<RawEnvironment<String>>(content).unwrap();

    assert_eq!(
        env,
        RawEnvironment {
            vars: indexmap::indexmap! {
                "ENV_VAR_1".to_string() => "env_var_1_value".to_string(),
            }
        }
    );
}

#[test]
fn test_include_environment_files() {
    #[derive(Debug, Deserialize, Clone)]
    pub struct Container {
        #[serde(rename(deserialize = "env_file"))]
        files: EnvFilesInclude,
    }

    let content: &str = r#"
env_file:
  - ./.env
  - /path/to/file_1
  - url: http://example.com
    checksum: "checksum"
  - file: /path/to/file_2
"#;

    let container: Container = serde_yaml::from_str::<Container>(content).unwrap();

    if let EnvFilesInclude::List(val) = container.files {
        if let EnvFilesInclude::Simple(variant) = &val[0] {
            assert_eq!(variant.to_str().unwrap(), "./.env".to_string());
        } else {
            unreachable!();
        }

        if let EnvFilesInclude::Simple(variant) = &val[1] {
            assert_eq!(variant.to_str().unwrap(), "/path/to/file_1".to_string());
        } else {
            unreachable!();
        }

        if let EnvFilesInclude::Url(UrlImport {
            url,
            ignore_errors: _,
            checksum: _,
        }) = &val[2]
        {
            assert_eq!(url.to_string(), "http://example.com".to_string());
        } else {
            unreachable!();
        }

        if let EnvFilesInclude::File(LocalFileImport {
            file,
            ignore_errors: _,
            path_resolve: _,
            checksum: _,
        }) = &val[3]
        {
            assert_eq!(file.to_str().unwrap(), "/path/to/file_2".to_string());
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
}

#[test]
fn test_include_remote() {
    let content: &str = r#"
url: http://example.com
checksum: "checksum"
"#;
    let container = serde_yaml::from_str::<EnvFilesInclude>(content).unwrap();

    if let EnvFilesInclude::Url(UrlImport {
        url,
        ignore_errors: _,
        checksum: _,
    }) = &container
    {
        assert_eq!(url.to_owned(), "http://example.com".to_string());
    } else {
        unreachable!()
    }
}

#[test]
fn test_include_file() {
    let content: &str = r#"
file: /path/to/file_2
"#;
    let container = serde_yaml::from_str::<EnvFilesInclude>(content).unwrap();

    if let EnvFilesInclude::File(LocalFileImport {
        file,
        ignore_errors: _,
        path_resolve: _,
        checksum: _,
    }) = &container
    {
        assert_eq!(file.to_str().unwrap(), "/path/to/file_2".to_string());
    } else {
        unreachable!();
    }
}

#[test]
fn test_include_simple_file() {
    let content: &str = r#"
./.env
"#;
    let container = serde_yaml::from_str::<EnvFilesInclude>(content).unwrap();

    if let EnvFilesInclude::Simple(variant) = container {
        assert_eq!(variant.to_str().unwrap(), "./.env".to_string());
    } else {
        unreachable!();
    }
}
