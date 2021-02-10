# Task automation tool on steroids for developers

**devrc** is an easy to use _task runner on steroids_ written in Rust.


[![Crates.io](https://img.shields.io/crates/v/devrc)](https://crates.io/crates/devrc)
[![Crates.io](https://img.shields.io/crates/d/devrc)](https://crates.io/crates/devrc)
[![CI](https://github.com/devrc-hub/devrc/workflows/CI/badge.svg?branch=master)](https://github.com/devrc-hub/devrc/actions)
![GitHub Workflow Status (branch)](https://img.shields.io/github/workflow/status/devrc-hub/devrc/CI/master)
[![GitHub](https://img.shields.io/github/license/devrc-hub/devrc)](https://github.com/devrc-hub/devrc/blob/master/LICENSE)


##  Overview

---

The **devrc** is a small and fast utility written in Rust.

It's userful for project or common routine automation such as minification, compilation, unit testing, linting and many more.

It's just single binary and you don't need to install `python`, `ruby` or something else.

It allows you to run tasks that are stored in YAML file named `Devrcfile`. Just type command, e.g. `devrc task_name_1 task_name_2 task_name_3`.

---

### Quick introduction

There are several ways to define task. Here are general variants:

1. Simple command definition without documentation string.
    ```yaml
    task_name: echo "Hello world"
    ```

2. More complex definition with documentation string.
    ```yaml
    task_name:
      desc: "Task description"
      exec: echo "Hello world"
    ```

To run task just type command `devrc TASK_NAME`, e.g.:

```bash
devrc task_name
```

For more details look into [usage section](#usage) or [examples](https://github.com/devrc-hub/devrc/blob/master/examples/).


### Why YAML is used?

There are many formats (e.g. TOML, Makefile, YAML, custom formats) that have been examining after many years of using `Makefiles` for project routine automation.

What are the benefits of using YAML for this purpose and why it's choosen:

1. YAML is designed to be _**human-friendly**_ and _**easy to read**_;
2. Syntax _**highlight works out of the box**_;
3. YAML is used _**industry-wide**_ for declarative configuration. For example, it's used by gitlab-ci, GitHub actions and ansible;
4. Many text editors and platforms have plugins or built-in tools to check YAML configuration syntax for you.
5. Good parsers for YAML already exist and I don't need to waste time for implementing and testing a self-written parser.



### Features

Lets start with an overview of features that exist in devrc:

  * [x] All tasks can be listed from the command line
  * [x] Template engine and variables interpolation are supported
  * [x] [Environment variables](#environment-variables) customization
  * [x] Command line completion scripts
  * [x] devrc supports [dotenv files](#dotenv-files)
  * [x] Writing task commands in different languages




## Table of contents

* [Overview](#overview)
    * [Why YAML is used?](#why-is-yaml-used)
    * [Quick introduction](#quick-introduction)
    * [Features](#features)
* [Installation](#installation)
    * [Install from crates](#install-from-crates.io)
    * [Compile from sources](#compile-from-sources)
    * [Binary Releases](#binary-releases)

* [Usage](#usage)
    * [Task definition](#task-definition)
    * [Template engine](#template-engine)
    * [Execution and compututation rules](#execution-and-computation-rules)
    * [Variables](#variables)
    * [Environment varibles](#environment-variables)
    * [Dotenv files](#dotenv-files)
    * [Writing task in different languages](#writing-task-commands-in-different-languages)

* [Contributing, feedback and suggestions](#contributing)

## Installation


To install `devrc`, you can download a [pre-compiled binary](#binary-releases), or you can [compile it from source](#compile-from-sources).
You may be able to install `devrc` using your OS’s package manager, depending on your platform.


### Install from crates.io

`devrc` is written in Rust. You will need rustc version 1.48.0 or higher.

The recommended way to install Rust for development is from the [official download page](https://www.rust-lang.org/tools/install), using rustup.

If you have the Rust toolchain already installed on your local system, you can use the `cargo install` command:

```bash
rustup update stable
cargo install devrc
```

Cargo will build the `devrc` binary and place it in `$HOME/.cargo`.


### Compile from sources

![GitHub](https://img.shields.io/badge/rustc-1.48+-lightgray.svg) ![GitHub](https://img.shields.io/github/license/devrc-hub/devrc)

Clone the repository and change it to your working directory.

```bash
git clone https://github.com/devrc-hub/devrc.git
cd devrc

rustup update stable
cargo install
```


### Binary Releases

Binary releases are available in the [github releases page](https://github.com/devrc-hub/devrc/releases) for macOS, and Linux targets.<br>
The following binaries are available for each release:

* x86_64-unknown-linux-musl
* x86_64-apple-darwin



## Usage

Tasks are stored as mappings (hashes/dictionaries/key-value pairs) in YAML file named `Devrcfile` by default.
Or you can specify environment variable `DEVRC_FILE` with alternative file name.
Also you can store global tasks in `~/.devrc` in your home directory or overwrite shared project tasks or varibles by local `Devrcfile.local` file.

The files loading process sequence:

1. Loading `~/.devrc` in home directory if it exists;
2. Loading files which are specified by command line option `-f`;
3. Loading `Devrcfile` or file name which is defined by environment variable `DEVRC_FILE` in the current directory;
4. Loading `Devrcfile.local`.

The name of the file is a case sensitive.


Task defition is like to definition of job in .gitlab-ci files. Key is used as task name and value is used to create task logic.
Some key names are reserved and described below.


### Task definition

There are different types of tasks.

1. Executable task
2. Configuration task (it hasn't been implemented yet).


There are three styles of task definition:

1. Simple command definition without documentation string.

    ```yaml
    task_name: echo "Hello world"
    ```
    or several commands in one task
    ```yaml
    task_name:
      - echo "Hello Alex!"
      - echo "Hello Alice!"
    ```

2. More complex definition with documentation string and variables.

    ```yaml
    variables:
      name: "Alex"
    task_name:
      desc: "Task description"
      exec: echo "Hello {{ name }}"
    ```

    or

    ```yaml
    variables:
      first_name: "Alex"
      second_name: "Alice"

    environment:
      ENV_NAME: "{{ second_name }}"

    task_name:
      desc: "Task description"
      exec:
        - echo "Hello {{ first_name }}!"
        - echo "Hello $ENV_NAME!"
    ```


If we write code to `Devrcfile` and type command `devrc task_name` in console it will output to console:

```text
Hello Alex!
Hello Alice!
```

Pay attention that `{{ first_name }}` are replaced by `Alex` by template engine and `$ENV_NAME!` are replaced by `Alice` by bash.

More complex examples can be found in [examples](https://github.com/devrc-hub/devrc/blob/master/examples/) directory.

### Variables

Variables are used by template engine to compute commands, another variables (global or local) or environment variables.

### Environment variables

### Dotenv files support


### Execution and computation rules

`devrc` views commands, global or local task defined variables, global or local task defines environment variables as templates. It applies template engine for commands before executing them or before variable assignment.

`devrc` consistently reads variables from files and applies a template engine based on Jinja2/Django syntax.

In the example text `variable_1 variable_2` is assigned to a variable `var_2` and text `env_var variable_1 variable_2` is assignet to an environment variable `ENV_VAR`:

```yaml
variables:
    var_1: "variable 1"
    var_2: "variable_2 {{ var_1}}"

environment:
    ENV_VAR: "env_var {{ var_2 }}"
```


### Template engine

### Writing task commands in different languages


```yaml

hello_1:
  desc: "Execute python script"
  exec: |
    #!/usr/bin/env python

    print("Hello from python")


hello_2:
  desc: "Execute javascript by node"
  exec: |
    #!/usr/bin/env node

    console.log("Hello from node")

```

Command `devrc hello_1 hello_2` output:
```text
Hello from python
Hello from node
```


## Alternatives

* Bash or makefile :-)
* [just](https://github.com/casey/just) - is a handy way to save and run project-specific commands. Commands are stored in a file called justfile with syntax inspired by `make`.
* [robo](https://github.com/tj/robo) - Simple YAML-based task runner written in Go. It looks abandoned.
* [go-task](https://github.com/go-task/task) - simpler Make alternative written in Go. It use Go's template engine which syntax makes me cry.


## Contributing

Any suggestion, feedback or contributing is highly appreciated.

I'm especially very thankful for your grammar correction contributions, because English isn't my native language.

Thank you for your support!
