# Autocommit

[![GitHub license](https://img.shields.io/github/license/sabry-awad97/autocommit)](https://github.com/sabry-awad97/autocommit/blob/main/LICENSE)

🚀 Autocommit is a CLI tool that helps you create professional and meaningful commits with ease. It uses 🤖 AI to generate impressive commit messages in seconds, so you can take control of your code history and make it shine!

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

Use the `config` command to set your preferences for autocommit.

```shell
autocommit config
```

Use the `commit` command to create a new commit.

```bash
autocommit commit
```

## 🛠️ Config Subcommand

It allows users to retrieve, modify and reset configuration values that are automatically saved to a configuration file on the user's machine.
This command supports several sub-commands, each with its own set of arguments:

### 📜 get

The `get` sub-command retrieves the values of one or more configuration keys.
If no keys are provided, all configuration keys and their respective values are printed to the console.

```sh
autocommit config get -h
```

### 📐 set

The set sub-command allows you to modify the values of one or more configuration keys. You can set multiple keys by providing a key-value pair for each one.

```sh
autocommit config set -h
```

### 🔄 reset

The `reset` sub-command resets all configuration values to their default values.

```sh
autocommit config reset -h
```

### 🌐 env

The `env` sub-command prints the configuration values as environment variables, which can be used in shell scripts.

```sh
autocommit config env -h
```

### 🚪 Config Keys

Here are the available configuration keys that can be used with autocommit-config:

- `description`: a boolean value that determines whether or not the user can add a description to commits.
- `emoji`: a boolean value that determines whether or not emojis are allowed in commit messages.
- `language`: a string that specifies the default language for commit messages.
- `name`: a string that stores the name of the Git user.
- `email`: a string that stores the email address of the Git user.
- `open_ai_api_key`: a string that stores an API key for OpenAI models.
- `api_host`: a string that specifies the host address for the OpenAI API.
- `open_ai_model`: an optional string that specifies the OpenAI model to use.

### 🌟 Examples

To retrieve all configuration values:

```sh
autocommit config get
```

To retrieve the values of specific keys:

```sh
autocommit config get --keys name email
```

To set the value of a configuration key:

```sh
autocommit config set emoji=true
```

To set multiple configuration keys:

```sh
autocommit config set emoji=true description=false
```

To reset all configuration values:

```sh
autocommit config reset
```

To print the configuration values as environment variables:

```sh
autocommit config env
```

To print the configuration values as environment variables for a specific shell:

```sh
autocommit config env --shell=bash
```

## ✨ Commit Subcommand

👉 To use Autocommit, navigate to the root directory of your Git repository and run the following command:

```sh
autocommit commit
```

🚀 This will start the Autocommit process, which will prompt you to select the files you want to stage, generate a commit message with AI, and push changes to a remote repository.

### 🔧 Configuration

Autocommit can be configured using a configuration file located at `~/.autocommit.toml`.

### 🙈 Ignore files

To exclude files from being submitted to OpenAI, create a `.autoignore` file in your main directory. Here's an example:

```sh
/target
.env
**/*.jpg
```

By default, autocommit ignores files matching: `*-lock.*` and `*.lock`

## 🤝 Contributing

Thank you for considering contributing to `autocommit`!.

🐛 If you find a bug or would like to request a new feature, please open an issue on GitHub.

👨‍💻 If you would like to contribute code, follow these steps to contribute:

1. Fork this repository and clone it to your local machine.
2. Create a new branch for your changes: `git checkout -b my-feature-branch`.
3. Make your changes and ensure that the tests pass: `cargo test`.
4. Commit your changes with a descriptive commit message: `git commit -m "feat: Add new feature"`.
5. Push your changes to the remote branch: `git push origin my-feature-branch`.
6. Create a new pull request and describe your changes.

When submitting a pull request, please include a detailed description of the changes you made, along with any relevant code comments or documentation. Additionally, please make sure that your changes are fully tested and do not break any existing functionality.

📝 To run the test suite, you can use the following command:

```sh
cargo test
```

This command will run all unit tests in the project and report any failures or errors.

👀 Your code will be reviewed by other contributors, and if accepted, will be merged into the main branch.

## 📄 License

This program is licensed under the [MIT License](https://opensource.org/licenses/MIT). Feel free to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of this program, subject to the conditions of the license.
