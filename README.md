# Tree for AI

A small **Rust CLI** for generating clean, **LLM-friendly project trees** for AI-assisted software development.

It prints relevant project files only, keeps the output content-free, respects Git where possible, and helps AI tools understand a codebase structure without unnecessary noise.

## What this project demonstrates

- Practical Rust CLI development with a focused developer-tooling use case.
- AI-assisted software workflow design: preparing clean project context for LLM prompts.
- Git-aware file discovery with safe fallbacks.
- Security-conscious defaults: file names and paths only, no file contents.
- Cross-platform usability for Windows, macOS and Linux.
- Simple automation support through JSON output.

## Why

- **Less noise, more signal:** hides build artifacts, caches and dependency folders.
- **Git-aware:** respects `.gitignore` by using `git ls-files` and falls back to a filesystem walk when Git is not available.
- **Safe by default:** prints only **file and folder names**. It never prints file contents.
- **Useful for AI prompts:** gives AI assistants enough project structure to reason about a codebase without pasting large amounts of unnecessary context.

## Features

- Clean 5-space indentation.
- Directories first, sorted alphabetically.
- Files sorted alphabetically.
- Relevance filter for source, config and text files.
- Optional inclusion of assets such as images, fonts and media.
- Optional inclusion of binary files.
- Optional listing of ignored files while still keeping the output content-free.
- Optional hiding of files that look like secrets.
- JSON output mode for scripting and automation.
- Works on Windows, macOS and Linux.

## Install

From a local checkout:

```bash
cargo install --path .
```

Install directly from GitHub:

```bash
cargo install --git https://github.com/lumlich/tree-for-ai
```

## Usage

Basic:

```bash
tree_for_ai
```

Popular options:

```bash
tree_for_ai --max-depth 4               # shorter tree
tree_for_ai --no-header                 # just the tree, no LLM helper header
tree_for_ai --include-assets            # also list assets such as images, fonts and media
tree_for_ai --hide-secrets              # do not even list files that look like secrets
tree_for_ai --json                      # machine-readable output
tree_for_ai --no-git --root .           # stay in the current folder, useful for monorepos
tree_for_ai --max-files 300             # cap the number of files
```

Redirect to a file or clipboard:

```bash
tree_for_ai --max-depth 4 > AI_TREE.txt
```

macOS:

```bash
tree_for_ai --max-depth 4 | pbcopy
```

Windows PowerShell:

```powershell
tree_for_ai --max-depth 4 | Set-Clipboard
```

Linux:

```bash
tree_for_ai --max-depth 4 | xclip -selection clipboard
```

## Options

- `--root <PATH>` – project root. Defaults to the Git root, otherwise the current working directory.
- `--no-git` – force filesystem mode and ignore Git.
- `--include-ignored` – also list `.gitignore`d files by name only.
- `--hide-secrets` – hide files that look like secrets, such as `.env` or `secrets.*`.
- `--include-assets` – include images, fonts and media files by name.
- `--include-binaries` – include all binary files. Not recommended for most AI prompts.
- `--max-depth <N>` – limit tree depth.
- `--max-files <N>` – limit the number of files after filtering.
- `--no-header` – do not print the LLM helper header.
- `--json` – output JSON instead of text.

## Security & Privacy

The tool prints **names and paths only**. It never prints file contents.

By default, it may include the **names** of common secret-related files, such as `.env`, so AI tools know that such files exist in the project structure. Use `--hide-secrets` to suppress even those names.

This makes the tool suitable for preparing project context for AI assistants without exposing source code, configuration values or secrets.

## How it works

1. Determine the project root via `git rev-parse --show-toplevel`, unless `--no-git` is used.
2. Collect relevant files using `git ls-files --cached --others --exclude-standard`.
3. Optionally include ignored files or common secret-related file names.
4. Fall back to a filtered filesystem walk when Git is not available.
5. Render a clean, LLM-friendly tree with 5-space indentation.
6. Optionally output the result as JSON for automation.

## Example output

```text
project-root
     src
          main.rs
          cli.rs
          tree.rs
     Cargo.toml
     README.md
```

## Compatibility

- Requires a working `git` binary for Git-aware mode.
- Falls back to filesystem walking when Git is not available.
- Tested on Windows, macOS and Linux with stable Rust.
- Uses lightweight dependencies: `clap`, `walkdir`, `regex` and `serde`.

## Project scope

This project is intentionally small and focused.

The goal is not to replace full documentation tools or code indexers. The goal is to produce a clean, readable and safe project tree that can be pasted into AI prompts or used in simple automation workflows.

## License

**MIT** — see [LICENSE](LICENSE).

## Contributing

Issues and pull requests are welcome.

Please keep the scope tight: small, fast, safe by default and focused on being copy-paste friendly for AI-assisted development.

## Changelog

See [RELEASE_NOTES_v0.1.0.md](RELEASE_NOTES_v0.1.0.md).
