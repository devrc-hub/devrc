// use devrc::variables::*;
// use indexmap::indexmap;

// #[test]
// fn test_environment_des_variant_1(){

//     let content = r#"var: var_1_value
// "#;

//     let vars: Variables = serde_yaml::from_str::<Variables>(content).unwrap();

//     assert_eq!(vars, Variables{vars: indexmap!{
//         "var".to_string() => Value(ValueVariant::String("var_1_value".to_string())),
//     }});
// }
