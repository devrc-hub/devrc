use devrc::{
    include::{IncludeFilesWrapper},
};
use std::fmt::Debug;

use serde::Deserialize;
use serde_yaml;


#[test]
fn test_empty_include() {
    #[derive(Debug, Deserialize, Clone)]
    pub struct Container {
        #[serde(default)]
        #[serde(rename(deserialize = "include"))]
        pub include_files: IncludeFilesWrapper,
    }

    let content: &str = r#"

some: value
"#;

    let container: Container = serde_yaml::from_str::<Container>(content).unwrap();

    // dbg!(container);

    dbg!(container.include_files.0);

    // assert_eq!(container.include_files.0, IncludeFiles::Empty);
}
