# Tree for AI

A tiny **Rust CLI** that prints an **LLM‑friendly project tree** — relevant files only, with clean 5‑space indentation. Ideal for pasting into prompts so AI tools understand your project structure without noise.

## Why
- **Less noise, more signal:** hides build artifacts, caches and dependency folders.
- **Git‑aware:** respects `.gitignore` (uses `git ls-files`) and falls back to a filesystem walk when Git isn't available.
- **Safe by default:** prints only **file and folder names** (no contents). Can show the existence of typical secrets (e.g. `.env`) by name if you want that context.

## Features
- 5‑space indentation, directories first (sorted), then files (sorted).
- Relevance filter for source/config/text files.
- Optional inclusion of assets (images, fonts, media) or binaries.
- Optional listing of ignored files, while still keeping it content‑free.
- JSON output mode for scripting/automation.
- Works on Windows, macOS, Linux.

## Install
From a local checkout:
```bash
cargo install --path .
```

(After publishing to GitHub, you can also install directly from the repo:)
```bash
cargo install --git https://github.com/<your-username>/<repo-name>
```

## Usage
Basic:
```bash
tree_for_ai
```

Popular options:
```bash
tree_for_ai --max-depth 4               # shorter tree
tree_for_ai --no-header                 # just the tree, no LLM header
tree_for_ai --include-assets            # also list assets (images, fonts, media)
tree_for_ai --hide-secrets              # do NOT even list files that look like secrets
tree_for_ai --json                      # machine-readable output
tree_for_ai --no-git --root .           # stay in the current folder (monorepos)
tree_for_ai --max-files 300             # cap the number of files
```

Redirect to a file or clipboard if you need:
```bash
tree_for_ai --max-depth 4 > AI_TREE.txt
# macOS
tree_for_ai --max-depth 4 | pbcopy
# Windows (PowerShell)
tree_for_ai --max-depth 4 | Set-Clipboard
# Linux
tree_for_ai --max-depth 4 | xclip -selection clipboard
```

## Options
- `--root <PATH>` – project root (defaults to Git root, else CWD)
- `--no-git` – force filesystem mode (ignore Git)
- `--include-ignored` – also list `.gitignore`d files (names only)
- `--hide-secrets` – hide files that *look* like secrets (e.g. `.env`, `secrets.*`)
- `--include-assets` – include images/fonts/media by name
- `--include-binaries` – include all binaries (not recommended)
- `--max-depth <N>` – limit tree depth
- `--max-files <N>` – limit number of files after filtering
- `--no-header` – do not print the LLM helper header
- `--json` – JSON output (instead of text)

## Security & Privacy
- The tool prints **names and paths only**, never file contents.
- By default it may include the **names** of common secrets (e.g. `.env`) so AI tools know they exist. Use `--hide-secrets` to suppress even the names.

## How it works (TL;DR)
1. Determine project root via `git rev-parse --show-toplevel` (unless `--no-git`).
2. Collect relevant files using `git ls-files --cached --others --exclude-standard`.
3. Optionally include ignored files or common secret names.
4. Fallback to a filtered filesystem walk when Git isn't available.
5. Render a clean tree with 5‑space indentation.

## Compatibility
- Requires a working `git` binary for Git‑aware mode.
- Tested on Windows, macOS and Linux with stable Rust.
- No heavy dependencies: `clap`, `walkdir`, `regex`, `serde`.

## License
**MIT** — see [LICENSE](LICENSE).

## Contributing
Issues and PRs are welcome. Please keep the scope tight: small, fast, and focused on being copy‑paste friendly for AI prompts.

## Changelog
See [RELEASE_NOTES_v0.1.0.md](RELEASE_NOTES_v0.1.0.md).
