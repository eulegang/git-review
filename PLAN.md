# Git Diff Viewer Plan

## Goal

Build a small Rust application for viewing Git diffs in a readable, navigable way.
Start with a command-line/TUI-oriented MVP, then expand toward richer review workflows.

## Target Users

- Developers reviewing local changes before committing.
- Developers inspecting branch-to-branch or commit-to-commit changes.
- Maintainers doing lightweight code review from a terminal.

## MVP Scope

### Inputs

Support viewing diffs from:

- Working tree changes: `git diff`
- Staged changes: `git diff --staged`
- Commit/range arguments: `git diff <rev>` or `git diff <base>..<head>`

### Core Features

- Detect current Git repository.
- Use libgit2 for git data source
- Render a readable diff view.
- Provide basic navigation:
  - Next/previous file
  - Next/previous hunk
  - Scroll within current diff
- Highlight additions, deletions, headers, and hunk metadata.

### Initial Interface

Start with a CLI command that prints a colorized diff summary/view.
Then evolve into an interactive TUI.

Example commands:

```sh
git-review                 # working tree diff
git-review --staged        # staged diff
git-review main..feature   # range diff
git-review HEAD~1          # compare with revision
```

## Non-MVP / Later Features

- Side-by-side diff view.
- Syntax highlighting inside changed lines.
- File tree/sidebar.
- Collapse/expand files and hunks.
- Inline comments or review notes.
- Export review notes.
- Open file at changed line in editor.

## Proposed Architecture

### Modules

- `cli`
  - Parse command-line arguments.
  - Decide which Git diff mode to run.
- `git`
  - Repository detection.
  - Git command execution.
  - Error handling around missing Git or invalid revisions.
- `render`
  - Non-interactive terminal rendering.
  - Color/styling abstraction.
- `tui` later
  - Interactive state and navigation.
  - Terminal event handling.

### Initial Data Model

```rust
struct Diff {
    files: Vec<FileDiff>,
}

struct FileDiff {
    old_path: Option<String>,
    new_path: Option<String>,
    hunks: Vec<Hunk>,
}

struct Hunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<DiffLine>,
}

enum DiffLine {
    Context(String),
    Added(String),
    Removed(String),
}
```

## Suggested Rust Dependencies

For MVP:

- `clap` for CLI parsing.
- `thiserror` for errors.
- `owo-colors` or `anstyle` for colored output.

For later TUI:

- `ratatui`

Optional later:

- `syntect` for syntax highlighting.

## Milestones

### Milestone 1: Basic CLI Diff Runner

- Add CLI parsing.
- Detect diff strategy variants.
- Print raw diff output.
- Handle common errors.

### Milestone 2: Parse Unified Diff

- Add structured diff parser.
- Unit test parser against sample diffs.
- Print parsed summary: changed files, hunk counts, line counts.

### Milestone 3: Colorized Linear Renderer

- Render parsed diff with colors.
- Add file headers and hunk headers.
- Preserve line prefixes.

### Milestone 4: Interactive TUI Prototype

- Display file list and diff pane.
- Keyboard navigation.
- Scroll support.
- Quit/help keys.

### Milestone 5: Review Quality Features

- Search.
- Expand/collapse hunks.
- Side-by-side view.
- Open in editor.

## Immediate Next Step

Implement Milestone 1:

1. Add `clap` and `eyre` dependencies.
2. Replace `src/main.rs` with a CLI that supports:
   - default working tree diff
   - `--staged`
   - optional revision/range args
3. Execute Git and print stdout.
4. Return useful errors when Git fails.
