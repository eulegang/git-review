use super::*;

fn temp_config(source: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "git-review-config-test-{}-{}.lua",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, source).unwrap();
    path
}

#[test]
fn parses_lua_config() {
    let path = temp_config(
        r##"
            return {
              theme = {
                added_bg = "#112233",
                selector_highlight_fg = "yellow",
                selector_highlight_bg = "black",
              },
              syntax = {
                paths = { "trees" },
                highlights = { ["function.builtin"] = "cyan", ["@string-special"] = "green" },
              },
            }
            "##,
    );

    let config = Config::load_file(&path).unwrap();

    assert_eq!(config.theme.added_bg.as_deref(), Some("#112233"));
    assert_eq!(config.syntax.highlights.len(), 2);

    let _ = std::fs::remove_file(path);
}

#[test]
fn parses_empty_lua_config() {
    let path = temp_config(
        r##"
            return { }
            "##,
    );

    Config::load_file(&path).unwrap();

    let _ = std::fs::remove_file(path);
}

#[test]
fn rejects_non_table_config() {
    let path = temp_config("return 1");

    assert!(Config::load_file(&path).is_err());

    let _ = std::fs::remove_file(path);
}
