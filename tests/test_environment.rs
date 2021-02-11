use devrc::environment::{EnvFile, EnvFilesWrapper, RawEnvironment, FileInclude, FileRemote, StringFileInclude};
use std::fmt::Debug;

use serde::Deserialize;
use serde_yaml;

#[macro_use]
use indexmap::indexmap;


#[test]
fn test_environment_des_variant_1() {
    let content = r#"ENV_VAR_1: env_var_1_value
"#;

    let env: RawEnvironment<String> = serde_yaml::from_str::<RawEnvironment<String>>(content).unwrap();

    assert_eq!(
        env,
        RawEnvironment {
            vars: indexmap! {
                "ENV_VAR_1".to_string() => "env_var_1_value".to_string(),
            }
        }
    );
}



#[test]
fn test_include_environment_files(){

    #[derive(Debug, Deserialize, Clone)]
    pub struct Container {
        #[serde(rename(deserialize = "env_file"))]
        files: EnvFilesWrapper,
    }


    let content: &str = r#"
env_file:
  - ./.env
  - /path/to/file_1
  - remote: http://example.com
  - file: /path/to/file_2
"#;

    let container: Container = serde_yaml::from_str::<Container>(content).unwrap();

    if let EnvFile::List(val) = container.files.0 {
        if let EnvFile::Simple(variant) = &val[0] {
            assert_eq!(variant.to_str().unwrap(), "./.env".to_string());
        } else {
            assert!(false);
        }

        if let EnvFile::Simple(variant) = &val[1] {
            assert_eq!(variant.to_str().unwrap(), "/path/to/file_1".to_string());
        } else {
            assert!(false);
        }

        if let EnvFile::Remote(FileRemote { remote }) = &val[2] {
            assert_eq!(remote.to_string(), "http://example.com".to_string());
        } else {
            assert!(false);
        }

        if let EnvFile::File(FileInclude { file, ignore_errors }) = &val[3] {
            assert_eq!(file.to_str().unwrap(), "/path/to/file_2".to_string());
        } else {
            assert!(false);
        }

    } else {
        assert!(false);
    }

}

#[test]
fn test_include_remote(){

    let content: &str = r#"
remote: http://example.com
"#;
    let container = serde_yaml::from_str::<EnvFile>(content).unwrap();

    if let EnvFile::Remote(FileRemote { remote }) = &container {
        assert_eq!(remote.to_owned(), "http://example.com".to_string());
    } else {
        assert!(false);
    }
}


#[test]
fn test_include_file(){

    let content: &str = r#"
file: /path/to/file_2
"#;
    let container = serde_yaml::from_str::<EnvFile>(content).unwrap();

    if let EnvFile::File(FileInclude { file, ignore_errors }) = &container {
        assert_eq!(file.to_str().unwrap(), "/path/to/file_2".to_string());
    } else {
        assert!(false);
    }

}

#[test]
fn test_include_simple_file(){

    let content: &str = r#"
./.env
"#;
    let container = serde_yaml::from_str::<EnvFile>(content).unwrap();

    if let EnvFile::Simple(variant) = container {
        assert_eq!(variant.to_str().unwrap(), "./.env".to_string());
    } else {
        assert!(false);
    }

}
