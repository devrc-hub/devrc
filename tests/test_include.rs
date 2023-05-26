use devrc::include::Include;
use std::fmt::Debug;

use serde::Deserialize;

#[test]
fn test_empty_include() {
    #[derive(Debug, Deserialize, Clone)]
    pub struct Container {
        #[serde(default)]
        #[serde(rename(deserialize = "include"))]
        pub include_files: Include,
    }

    let content: &str = r#"

some: value
"#;

    let container: Container = serde_yaml::from_str::<Container>(content).unwrap();

    // dbg!(container);

    dbg!(container.include_files);

    // assert_eq!(container.include_files.0, IncludeFiles::Empty);
}
