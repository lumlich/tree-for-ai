# Tree for AI v0.1.0 — Initial public release

**Date:** 2025-09-30

### Highlights
- Git‑aware, LLM‑friendly tree for your project (5‑space indentation).
- Relevance filter for source/config/text files; directories first, files second (both sorted).
- Optional inclusion of assets, ignored files, or all binaries.
- JSON mode for scripting.
- Prints **names only** (no file contents).

### Options (recap)
- `--root <PATH>`, `--no-git`
- `--include-ignored`, `--hide-secrets`, `--include-assets`, `--include-binaries`
- `--max-depth <N>`, `--max-files <N>`
- `--no-header`, `--json`

### Known limitations
- The relevance whitelist is opinionated (sane defaults). You can opt into assets/binaries.
- Requires `git` for the Git‑aware fast path (falls back to a filesystem walk).

### Thanks
Thanks to early testing for surfacing common workflow needs around AI prompting and project structure.
