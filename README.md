Tasks automation tool for developers
====================================

**devrc** is an easy to use YAML-based task runner written in Rust.


Tasks are stored in YAML file named Devrcfile by default. It's very similar to .gitlab-ci file.
All tasks are stored as mappings (hashes/dictionaries/key-value pairs).
Key is used as task name and value is used to create task logic.
Some key names are reserved and described below.


Features
--------

Lets start with an overview of features that exist in devrc:

  * [x] All tasks can be listed from the command line



Use cases
-----------

Why?
----


Installation
------------


## Install from crates.io

If you have the Rust toolchain already installed on your local system.

``` shell
rustup update stable
cargo install devrc
```


## Compile and run from sources

Clone the repository and change it to your working directory.

```shell
git clone https://github.com/devrc-hub/devrc.git
cd devrc

rustup override set stable
rustup update stable
cargo install
```

# Getting started


## Listing tasks


## Running tasks


## Variables


## Environment variables


## Architecture

### Contributing
Any suggestion, feedback or contributing is highly appreciated. Thank you for your support!
