use clap::{ArgAction, Parser};
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::{DirEntry, WalkDir};

const INDENT_SPACES: usize = 5;

#[derive(Parser, Debug)]
#[command(
    name = "Tree for AI",
    version,
    about = "LLM‑friendly project tree (Rust MVP)"
)]
struct Args {
    /// Project root (defaults to Git root if available, otherwise CWD)
    #[arg(long)]
    root: Option<PathBuf>,

    /// Force filesystem mode (ignore Git)
    #[arg(long)]
    no_git: bool,

    /// Also include .gitignore'd files (names only)
    #[arg(long)]
    include_ignored: bool,

    /// Hide files that look like secrets (.env, secrets.*)
    #[arg(long, action = ArgAction::SetTrue)]
    hide_secrets: bool,

    /// Include common assets (images, fonts, media)
    #[arg(long, action = ArgAction::SetTrue)]
    include_assets: bool,

    /// Include all binaries (not recommended)
    #[arg(long, action = ArgAction::SetTrue)]
    include_binaries: bool,

    /// Maximum depth (number of path segments after the root)
    #[arg(long)]
    max_depth: Option<usize>,

    /// Limit the number of files after filtering
    #[arg(long)]
    max_files: Option<usize>,

    /// Do not print the LLM helper header
    #[arg(long, action = ArgAction::SetTrue)]
    no_header: bool,

    /// Print JSON instead of a text tree
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct TreeNode {
    name: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    dirs: BTreeMap<String, TreeNode>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    files: Vec<String>,
}

impl TreeNode {
    fn new(name: String) -> Self {
        Self {
            name,
            dirs: BTreeMap::new(),
            files: Vec::new(),
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let start = args
        .root
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    // Pass a reference so we don't move `start`, then fall back to `start` if canonicalization fails
    let start = fs::canonicalize(&start).unwrap_or(start);

    // Prefer Git root when available (fast and respects .gitignore)
    let git_root = if args.no_git {
        None
    } else {
        detect_git_root(&start)
    };
    let root = git_root.clone().unwrap_or_else(|| start.clone());

    // Collect files (Git-aware first, else filesystem walk)
    let mut files = if git_root.is_some() && !args.no_git {
        list_files_git(&root, args.include_ignored, !args.hide_secrets)
            .unwrap_or_else(|_| list_files_fs(&root))
    } else {
        list_files_fs(&root)
    };

    // Relevance filter
    files.retain(|p| {
        is_relevant_path(
            p,
            &FilterOptions {
                include_assets: args.include_assets,
                include_binaries: args.include_binaries,
                hide_secrets: args.hide_secrets,
            },
        )
    });

    // Deterministic ordering and optional file cap
    files.sort_by(|a, b| a.cmp(b));
    if let Some(max) = args.max_files {
        if files.len() > max {
            files.truncate(max);
        }
    }

    let mut tree = build_tree(&files, &root, args.max_depth);

    let mode = if git_root.is_some() && !args.no_git {
        "git-aware"
    } else {
        "fs-heuristic"
    };

    if args.json {
        let payload = serde_json::json!({
            "root": root.to_string_lossy(),
            "mode": mode,
            "indent": INDENT_SPACES,
            "files_count": files.len(),
            "tree": tree,
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        return Ok(());
    }

    // Human-friendly header for LLMs
    let mut out = String::new();
    if !args.no_header {
        out.push_str("# Tree for AI\n");
        out.push_str(&format!("root: {}\n", root.to_string_lossy()));
        out.push_str(&format!("mode: {}\n", mode));
        out.push_str("rules:\n");
        out.push_str("- Work only with files/paths listed below unless explicitly asked to create new ones.\n");
        out.push_str("- All paths are relative to the root above.\n");
        out.push_str("- File contents are not included; ask if more context is needed.\n\n");
    }
    out.push_str(&render_tree_text(&mut tree, INDENT_SPACES, args.max_depth));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    print!("{out}");
    Ok(())
}

/// Return the Git root if we're inside a repo, otherwise None.
fn detect_git_root(cwd: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(PathBuf::from(s))
    }
}

/// List files from Git (tracked + untracked), optionally include ignored ones.
/// When `include_secret_names` is true, we also include names that look like secrets (e.g. `.env`),
/// but still only as names/paths (never contents).
fn list_files_git(
    root: &Path,
    include_ignored: bool,
    include_secret_names: bool,
) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    // Tracked + untracked (excluding ignored)
    let out = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .current_dir(root)
        .output()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout);
        for line in s.lines().filter(|l| !l.trim().is_empty()) {
            paths.push(root.join(line));
        }
    }

    // Optionally include ignored (and/or secret-like names)
    if include_ignored || include_secret_names {
        let out_ign = Command::new("git")
            .args(["ls-files", "--ignored", "--exclude-standard"])
            .current_dir(root)
            .output()?;
        if out_ign.status.success() {
            let s = String::from_utf8_lossy(&out_ign.stdout);
            for line in s.lines().filter(|l| !l.trim().is_empty()) {
                let p = root.join(line);
                if include_ignored || (include_secret_names && is_secret_path(&p)) {
                    paths.push(p);
                }
            }
        }
    }

    // Deduplicate & sort
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Filesystem fallback when Git isn't available (prunes well-known noisy folders)
fn list_files_fs(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| fs_dir_allow(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Directory filter for FS walk: skip dependency/build/cache folders to avoid noise.
fn fs_dir_allow(e: &DirEntry) -> bool {
    if e.depth() == 0 {
        return true;
    }
    let name = e.file_name().to_string_lossy().to_lowercase();
    let deny_dirs = [
        ".git",
        ".hg",
        ".svn",
        "__pycache__",
        ".cache",
        ".mypy_cache",
        ".pytest_cache",
        ".ruff_cache",
        ".tox",
        ".venv",
        "venv",
        "env",
        "node_modules",
        ".pnpm-store",
        "dist",
        "build",
        "out",
        ".next",
        ".nuxt",
        ".angular",
        ".parcel-cache",
        "target",
        "bin",
        "obj",
        ".gradle",
        ".idea",
        ".vscode",
        ".terraform",
        ".serverless",
        ".docusaurus",
    ];
    !deny_dirs.contains(&name.as_str())
}

struct FilterOptions {
    include_assets: bool,
    include_binaries: bool,
    hide_secrets: bool,
}

/// Returns true if a path is relevant for an AI-friendly project tree.
/// Defaults to source/config/text files; assets/binaries are opt-in.
fn is_relevant_path(p: &Path, opts: &FilterOptions) -> bool {
    if let Some(name) = p.file_name().and_then(OsStr::to_str) {
        let lower = name.to_lowercase();

        // Always-ignored junk files
        let deny_files = [".ds_store", "thumbs.db"];
        if deny_files.contains(&lower.as_str()) {
            return false;
        }

        // Locks and similar rarely help in AI context
        let lock_names = [
            "yarn.lock",
            "package-lock.json",
            "pnpm-lock.yaml",
            "pipfile.lock",
            "poetry.lock",
        ];
        if lower.ends_with(".lock") || lock_names.contains(&lower.as_str()) {
            return false;
        }
    }

    // If secrets are not hidden, show secret-like files by name
    if !opts.hide_secrets && is_secret_path(p) {
        return true;
    }

    if opts.include_binaries {
        return true;
    }

    // Relevance by extension / special filenames
    let relevant_exts = [
        // docs/config
        "md","rst","adoc","txt","json","jsonc","yaml","yml","toml","ini","cfg","conf","env","properties",
        // web
        "html","htm","css","scss","less",
        // code
        "rs","py","pyi","ipynb",
        "js","cjs","mjs","jsx","ts","tsx",
        "sh","bash","zsh","ps1","bat","cmd",
        "go","java","kt","kts",
        "c","h","cpp","hpp","cc","hh",
        "cs","vb","php","rb","swift","scala","erl","ex","exs",
        "sql","prisma","graphql","gql",
        "gradle","groovy","tf","sln","csproj","fsproj","vbproj","vcxproj",
        "editorconfig","gitattributes","gitignore","eslintignore","prettierignore",
    ];
    let asset_exts = [
        "png","jpg","jpeg","gif","svg","webp","ico","bmp","tiff",
        "mp3","wav","flac","mp4","mov","mkv","avi",
        "woff","woff2","eot","ttf","otf","pdf",
        "zip","tar","gz","tgz","bz2","7z","rar",
    ];

    // Dockerfile / Makefile without extension
    if let Some(stem) = p.file_name().and_then(OsStr::to_str) {
        let special = ["Dockerfile", "Makefile", "dockerfile", "Dockerfile.dev"];
        if special.contains(&stem) {
            return true;
        }
    }

    let ext = p
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_lowercase());

    if let Some(e) = &ext {
        if relevant_exts.contains(&e.as_str()) {
            return true;
        }
        if opts.include_assets && asset_exts.contains(&e.as_str()) {
            return true;
        }
    }

    false
}

/// Heuristic for "secret-like" names (names only; never contents).
fn is_secret_path(p: &Path) -> bool {
    let fname = p
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default()
        .to_lowercase();

    // .env, .env.local, .env.* and anything containing "secret"/"secrets" as a word
    if fname == ".env" || fname.starts_with(".env.") {
        return true;
    }
    let re = Regex::new(r"(?:^|[^a-z])secrets?(?:$|[^a-z])").unwrap();
    re.is_match(&fname)
}

/// Build a directory tree from a flat list of paths (respects max_depth for files).
fn build_tree(paths: &[PathBuf], root: &Path, max_depth: Option<usize>) -> TreeNode {
    let root_name = root
        .file_name()
        .and_then(OsStr::to_str)
        .map(|s| s.to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string());

    let mut tree = TreeNode::new(root_name);

    for p in paths {
        let rel = p.strip_prefix(root).unwrap_or(p);
        let mut cur = &mut tree;
        let parts: Vec<_> = rel.components().collect();
        for (i, comp) in parts.iter().enumerate() {
            let name = comp.as_os_str().to_string_lossy().to_string();
            let is_last = i == parts.len() - 1;

            if is_last {
                let depth = parts.len();
                if max_depth.map(|m| depth <= m).unwrap_or(true) {
                    cur.files.push(name);
                }
            } else {
                let depth = i + 1; // how many parts so far
                if max_depth.map(|m| depth > m).unwrap_or(false) {
                    break; // don't descend deeper
                }
                cur = cur
                    .dirs
                    .entry(name)
                    .or_insert_with_key(|k| TreeNode::new(k.clone()));
            }
        }
    }
    tree
}

/// Render the tree with 5‑space indentation. Directories first (sorted), then files (sorted).
fn render_tree_text(tree: &mut TreeNode, indent: usize, max_depth: Option<usize>) -> String {
    fn rec(
        n: &mut TreeNode,
        level: usize,
        indent: usize,
        lines: &mut Vec<String>,
        max_depth: Option<usize>,
    ) {
        if level == 0 {
            lines.push(format!("{}/", n.name));
        }
        // Directories
        for (_k, v) in &mut n.dirs {
            lines.push(format!("{}{}/", " ".repeat(indent * (level + 1)), v.name));
            if max_depth.map(|m| level + 1 < m).unwrap_or(true) {
                rec(v, level + 1, indent, lines, max_depth);
            }
        }
        // Files
        n.files
            .sort_unstable_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        for f in &n.files {
            lines.push(format!("{}{}", " ".repeat(indent * (level + 1)), f));
        }
    }
    let mut lines = Vec::new();
    rec(tree, 0, indent, &mut lines, max_depth);
    lines.join("\n")
}
