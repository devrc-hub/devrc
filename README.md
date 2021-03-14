# Task automation tool on steroids for developers

[![Crates.io](https://img.shields.io/crates/v/devrc)](https://crates.io/crates/devrc)
[![Crates.io](https://img.shields.io/crates/d/devrc)](https://crates.io/crates/devrc)
[![CI](https://github.com/devrc-hub/devrc/workflows/CI/badge.svg?branch=master)](https://github.com/devrc-hub/devrc/actions)
[![Security audit](https://github.com/devrc-hub/devrc/workflows/Security%20audit/badge.svg?branch=master)](https://github.com/devrc-hub/devrc/actions)
[![Minimum supported Rust version](https://img.shields.io/badge/rustc-1.48+-brightgreen.svg)](#compile-from-sources)
[![Lines Of Code](https://tokei.rs/b1/github/devrc-hub/devrc?category=code)](#contributing)
[![LICENSE](https://img.shields.io/github/license/devrc-hub/devrc)](https://github.com/devrc-hub/devrc/blob/master/LICENSE)


##  Overview

---

**devrc** is an easy to use _task runner on steroids_. It's a small and fast utility written in Rust 🦀.

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

### Features

Lets start with an overview of features that exist in devrc:

  * [x] All tasks can be listed from the command line with documentation
  * [x] Template engine and variables interpolation are supported
  * [x] [Environment variables](#environment-variables) customization
  * [x] Command line completion scripts
  * [x] devrc supports [dotenv files](#dotenv-files)
  * [x] Writing task commands in different languages
  * [x] Task parameters and user input
  * [ ] Remote command execution
  * [x] Read `Devrcfile` contents from stdin
  * [x] Global and local defined variables and environment variables
  * [x] Embedded [deno runtime](#embedded-deno-runtime)


### Why YAML is used?

There are many formats (e.g. TOML, Makefile, YAML, custom formats) that have been examining after many years of using `Makefiles` for project routine automation.

What are the benefits of using YAML for this purpose and why it's choosen:

1. YAML is designed to be _**human-friendly**_ and _**easy to read**_;
2. Syntax _**highlight works out of the box**_;
3. YAML is used _**industry-wide**_ for declarative configuration. For example, it's used by gitlab-ci, GitHub actions and ansible;
4. Many text editors and platforms have plugins or built-in tools to check YAML configuration syntax for you.
5. Good parsers for YAML already exist and I don't need to waste time for implementing and testing a self-written parser.


## Table of contents

* [Overview](#overview)
    * [Why YAML is used?](#why-yaml-is-used)
    * [Quick introduction](#quick-introduction)
    * [Features](#features)
* [Installation](#installation)
    * [Install from crates](#install-from-crates.io)
    * [Compile from sources](#compile-from-sources)
    * [Binary Releases](#binary-releases)
* [Usage](#usage)
    * [Task definition](#task-definition)
    * [Reserved keywords](#reserved-keywords)
    * [Config options](#configuration)
    * [Template engine](#template-engine)
    * [Execution and compututation rules](#execution-and-computation-rules)
    * [Variables](#variables)
    * [Environment varibles](#environment-variables)
    * [Dotenv files](#dotenv-files)
    * [Task dependencies](#task-dependencies)
    * [Writing task in different languages](#writing-task-commands-in-different-languages)
    * [Task parameters and user input](#task-parameters-and-user-input)
    * [Remote command execution](#remote-command-execution)
    * [Read `Devrcfile` from sdtin](#read-devrcfile-from-stdin)
    * [Embedded deno execution runtime](#embedded-deno-runtime)


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

![GitHub](https://img.shields.io/badge/rustc-1.48+-brightgreen.svg) ![GitHub](https://img.shields.io/github/license/devrc-hub/devrc)

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
* x86_64-unknown-linux-gnu
* x86_64-apple-darwin



## Usage

Tasks are stored as mappings (hashes/dictionaries/key-value pairs) in YAML file named `Devrcfile` by default.
Or you can specify environment variable `DEVRC_FILE` with alternative file name.
Also you can store global tasks in `~/.devrc` in your home directory or overwrite shared project tasks or varibles by local `Devrcfile.local` file.

If command line option `-f` is used:
1. Loading `~/.devrc` in home directory if it exists and if command line flag `-g` or option `devrc_config.global` are enabled;
2. Loading files which are specified by command line option `-f`;

If command line option `-f` isn't used:
1. Loading `~/.devrc` in home directory if it exists and if command line flag `-g` or option `devrc_config.global` in `Devrcfile` are enabled;
2. Loading `Devrcfile` or file name which is defined by environment variable `DEVRC_FILE` in the current directory;
3. Loading `Devrcfile.local`.

The names of the files are a case sensitive.

Task defition is like to definition of job in .gitlab-ci files. Key is used as task name and value is used to create task logic.
Some key names are reserved and described below.


### Task definition

There are different types of tasks.

1. Executable task
2. Configuration task (it hasn't been implemented yet).


There are three styles of task definition.

Simple command definition without documentation string:

```yaml
variables:
  name: "Alex"

task_name: echo "Hello {{ name }}"
```

or several commands in one task
```yaml

variables:
  first_name: "Alex"
  second_name: "Alice"

environment:
  ENV_NAME: "{{ second_name }}"

task_name:
  - echo "Hello {{ first_name }}!"
  - echo "Hello $ENV_NAME!"
```


This tasks can be rewritten to more complex definition form with documentation string and variables.

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


### Reserved keywords

There are few reserved keywords that can't be used as task name:

* `devrc_config` - [global options](#configuration) such as `shell`, `log_level`, `current_directory`;
* `variables` - global set of variables that are used by template engine;
* `environment` - global set of environment variables that are passed to children process's environment;
* `before_script` - is a task that are executed before first task;
* `after_script` - is a task that are executed after last task;
* `before_task` - is a task that are executed before each task;
* `after_task` - is a task that are executed after each task;
* `env_file` - is used for [dotenv files](#dotenv-files-support);


### Configuration


```yaml

devrc_config:
  global: true
  interpreter: /bin/bash -c
  default: [task_1, task_2]

```

### Variables

Variables are used by template engine to compute commands, another variables (global or local) or environment variables.
If there exists global and local variables with the same name, then local will overwrite it's value.

### Environment variables

Environment variables that are passed to children process's environment and they must be accessed using $VARIABLE_NAME in commands. Environment variables can be defined globally or locally in task. If there exists global and local environment variables with the same name, then local will overwrite it's value.
The shell will expand or substitute the value of a variable into a command line if you put a Dollar Sign `$` in front of the variable name.

```yaml

tast_name:
  environment:
    name: "Alex"

  exec: Hello $name!

```

### Dotenv files support

`devrc` can load environment variables from env (dotenv) files. These variables are environment variables, not template variables. By default if something goes wrong in dotenv loading, `devrc` will break and exit. You can change default behaviour by using option `ignore_errors` and if something goes wrong, `devrc` will continue.

If env file contains:
```text
ENV_NAME=Alex
```

you can load environment varibles from files using one of variants:

```yaml
env_file:
  - ./.env
  - file: ./.env_3
    ignore_errors: true
  - file: ./.env_2

task_name: echo "Hello $ENV_NAME"

```

File path can be absolute or relative. Part `./` substitute to current directory.


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

### Task dependencies

Task may have dependencies from another tasks. Dependencies of a task always run before a task execution and before `before_task` hook.
This is useful to make some job before given task, like clean cache, remove atrifacts, etc. Dependencies run in series.

```yaml

task_1: echo "Hello world!"

task_2:
  exec: echo "Hello $USER!"
  deps: [task_1]

```


### Template engine

### Task parameters and user input

Tasks may have parameters. Task arguments are passed after task name when devrc is called. Parameters can be required or have default value. Also parameter value is a template string and previously defined variables or parameters can be used:

```bash
devrc task_name arg1 arg2 "argument with spaces and {{ param1 }}" task_name2
```

There are 2 forms of parameters definitions.

Here is a simple form where `param1` and `param2` are required and `param2` is optional with default value:

```yaml
task_name param1 param2 param3="argument with spaces and {{ param1 }}": |
  echo "Hello {{ param1 }} and {{ param2 }}";
  echo "{{ param3 }}"

```

Default value must be in double quotes. If you need to use quotes inside default value you can escape it by `\` (backslash) symbol.


Here is a more complex form:

Here task has a required parameter `name`, an optional parameter `other` with default value of `Alice` and host parameter `host` which is assigned after user input.

```yaml

task_name:
  exec:
    - echo "Hello {{ name }} and {{ other}}"

  params:
    name: # this is required parameter
    other: "Alice"

```

Here usage example:

```bash
$ devrc task_name name="Alex"
Hello Alex and Alice
```

or

```bash
$ devrc task_name "Alex"
Hello Alex and Alice
```



### Remote command execution

It's also possible to execute task on remote hosts.

_Notice:_ This feature has't been implemented yet.

```yaml
task_name:
  exec: echo "Hello {{ name }} from $(hostname)"
  variables:
    name: "Alex"
    username: root

  remote:
    - "{{ username }}@hostname1:22"
    - root@hostname2:22
```

```yaml
task_name:
  exec: echo "Hello {{ name }} from $(hostname)"
  variables:
    name: "Alex"
    username: root

  remote:
    hosts:
      - "{{ username }}@hostname1:22"
      - hostname2
```

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

### Read Devrcfile from stdin

Instead of reading files `devrc` can read tasks file from stdin. To enable this behaviour pass `--stdin` flag:

```bash
cat ./Devrcfile | devrc --stdin task_name
```

or

```bash
devrc --stdin task_name < ./Devrcfile
```

### Embedded deno runtime

Devrc has embedded [deno runtime](https://github.com/denoland/deno). Deno is a simple, modern and secure runtime for JavaScript and TypeScript that uses V8. This runtime can be enabled on a global level or task level. By default all permissions disabled. No file, network, or environment access, unless explicitly enabled.


```yaml

devrc_config:
  interpreter:
    runtime: deno-runtime
    permissions:
      - allow-env
      - allow-net: [google.com, httpbin.org]

      # - disable-all
      # - allow-all
      # - allow-env
      # - allow-hrtime
      # - allow-net: [google.com, httpbin.org]
      # - allow-plugin
      # - allow-read: ["/tmp"]
      # - allow-run
      # - allow-write-all
      # - allow-write: ["/tmp"]

colors: |
  import { bgBlue, bold, italic, red } from "https://deno.land/std/fmt/colors.ts";

  const name = prompt("What is your name?");

  confirm(`Are you sure ${name} is your name?`);

  if (import.meta.main) {
     console.log(bgBlue(italic(red(bold(`Hello ${name} !`)))));
  }

```

More examples can be found [here](https://github.com/devrc-hub/devrc/blob/master/examples/deno_usage.yml).



## Alternatives

* Bash or makefile :-)
* [just](https://github.com/casey/just) - is a handy way to save and run project-specific commands. Commands are stored in a file called justfile with syntax inspired by `make`.
* [robo](https://github.com/tj/robo) - Simple YAML-based task runner written in Go. It looks abandoned.
* [go-task](https://github.com/go-task/task) - simpler Make alternative written in Go. It uses Go's template engine which syntax makes me cry.


## Contributing

Any suggestion, feedback or contributing is highly appreciated.

I'm especially very thankful for your grammar correction contributions, because English isn't my native language.

Thank you for your support!
