# Autocommit

[![GitHub license](https://img.shields.io/github/license/sabry-awad97/autocommit)](https://github.com/sabry-awad97/autocommit/blob/main/LICENSE)

Autocommit is a CLI tool that helps you create professional and meaningful commits with ease. It uses AI to generate impressive commit messages in seconds, so you can take control of your code history and make it shine!

## Installation

1. Install Rust (<https://www.rust-lang.org/tools/install>)
2. Clone the repository and navigate to its root directory
3. Install Git from the [official website](https://git-scm.com/downloads).
4. Once you have Git installed, you can install Autocommit using Cargo [Cargo](https://doc.rust-lang.org/cargo/), the Rust package manager:

   ```shell
   cargo install --path .
   ```

## Usage

After Autocommit is installed, you can use the autocommit command to set up automatic commits for your Git repositories.

```shell
autocommit [SUBCOMMAND]
```

### Subcommands

The `config` subcommand

Use the `config` command to set your preferences for autocommit.

```shell
autocommit config
```

The `commit` subcommand

Use the `commit` command to create a new commit.

```bash
autocommit commit
```
