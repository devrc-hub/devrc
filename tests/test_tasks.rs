use devrc::{tasks::*, variables::*};
use indexmap::indexmap;



// #[test]
// fn test_string_or_struct() {
//     let content = r#"
// task1: echo "command as simple string"
// task2: echo "command as simple string2"
// task3:
//    cmd: echo "command as dict"
// "#;


//     let tasks = serde_yaml::from_str::<Tasks>(content).unwrap();

//     // dbg!(tasks);
// }


#[test]
fn test_simple_string_task(){

    let content: &str = r#"

command_name: command value
"#;

    let container: Tasks = serde_yaml::from_str::<Tasks>(content).unwrap();

    dbg!(container);

}


#[test]
fn test_multiline_task(){

    let content: &str = r#"

simple: |
  echo "Command 1"
  echo "Command 2"
"#;

    let container: Tasks = serde_yaml::from_str::<Tasks>(content).unwrap();

    dbg!(container);

}

#[test]
fn test_extented_task_syntax_1(){

    let content: &str = r#"
# execute bash command
bash:
  variables:
    local_task_variable: "local task varible value"
  exec: echo "Hello world"
"#;

    let container: Tasks = serde_yaml::from_str::<Tasks>(content).unwrap();

    dbg!(container);

}

#[test]
fn test_extented_task_syntax_2(){

    let content: &str = r#"
task_3:
  exec:
    - echo "Task 3 first command"
    - exec": echo 'Task 3 second command'
      register: output

    - echo "Output"
"#;

    let container: Tasks = serde_yaml::from_str::<Tasks>(content).unwrap();

    dbg!(container);

}


#[test]
fn test_extented_task_syntax_with_deps(){

    let content: &str = r#"
task_with_deps:
  deps:
    - task1
    - task2
"#;

    let container: Tasks = serde_yaml::from_str::<Tasks>(content).unwrap();

    dbg!(container);

}



#[test]
fn test_executable_script(){

    let content: &str = r#"
# execute bash command
bash:
  variables:
    local_task_variable: "local task varible value"
  exec: echo "Hello world"
"#;

    let container: Tasks = serde_yaml::from_str::<Tasks>(content).unwrap();

    dbg!(container);

}


#[test]
fn test_extented_task_syntax_with_shebang(){

    let content: &str = r#"
task3:
  exec: |
    #!/usr/bin/env python

    print("Hello world")
"#;

    let container: Tasks = serde_yaml::from_str::<Tasks>(content).unwrap();

    dbg!(container);

}