use eyre::{Context, Result, bail};
use git2::{Config as GitConfig, Repository};
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub added_bg: Color,
    pub removed_bg: Color,
    pub binary_bg: Color,
    pub selected_modifier: Modifier,
    pub selector_highlight: Style,
    pub warning: Style,
}

impl Theme {
    pub fn load(repo: &Repository) -> Result<Self> {
        let git_config = repo.config().context("failed to open Git config")?;
        let mut theme = Self::default();

        if let Some(color) = git_config.get_color("git-review.theme.added-bg") {
            theme.added_bg = color;
        }

        if let Some(color) = git_config.get_color("git-review.theme.removed-bg") {
            theme.removed_bg = color;
        }

        if let Some(color) = git_config.get_color("git-review.theme.binary-bg") {
            theme.binary_bg = color;
        }

        if let Some(color) = git_config.get_color("git-review.theme.selector-highlight-fg") {
            theme.selector_highlight = theme.selector_highlight.fg(color);
        }

        if let Some(color) = git_config.get_color("git-review.theme.warning-fg") {
            theme.warning = theme.warning.fg(color);
        }

        Ok(theme)
    }

    pub const fn new() -> Self {
        Self {
            added_bg: Color::Green,
            removed_bg: Color::Red,
            binary_bg: Color::Gray,
            selected_modifier: Modifier::BOLD.union(Modifier::REVERSED),
            selector_highlight: Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            warning: Style::new().fg(Color::Yellow),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_color(raw: &str) -> Result<Color> {
    let color = raw.trim().to_ascii_lowercase();

    if color == "reset" || color == "default" {
        return Ok(Color::Reset);
    }

    if let Some(hex) = color.strip_prefix('#') {
        if hex.len() != 6 {
            bail!("hex colors must be in #rrggbb format");
        }

        let red = u8::from_str_radix(&hex[0..2], 16).context("invalid red channel")?;
        let green = u8::from_str_radix(&hex[2..4], 16).context("invalid green channel")?;
        let blue = u8::from_str_radix(&hex[4..6], 16).context("invalid blue channel")?;

        return Ok(Color::Rgb(red, green, blue));
    }

    if let Ok(index) = color.parse::<u8>() {
        return Ok(Color::Indexed(index));
    }

    match color.as_str() {
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" | "grey" => Ok(Color::Gray),
        "dark-gray" | "dark-grey" => Ok(Color::DarkGray),
        "light-red" => Ok(Color::LightRed),
        "light-green" => Ok(Color::LightGreen),
        "light-yellow" => Ok(Color::LightYellow),
        "light-blue" => Ok(Color::LightBlue),
        "light-magenta" => Ok(Color::LightMagenta),
        "light-cyan" => Ok(Color::LightCyan),
        "white" => Ok(Color::White),
        _ => bail!("unknown color {raw:?}"),
    }
}

trait Colorful {
    fn get_color(&self, name: &str) -> Option<Color>;
}

impl Colorful for GitConfig {
    fn get_color(&self, name: &str) -> Option<Color> {
        if let Ok(value) = self.get_string(name) {
            parse_color(&value).ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> GitConfig {
        let path = std::env::temp_dir().join(format!(
            "git-review-test-{}-{}.config",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&path);

        GitConfig::open(&path).unwrap()
    }

    #[test]
    fn applies_individual_color_property_from_git_config() {
        let mut config = config();
        config
            .set_str("git-review.theme.added-bg", "#112233")
            .unwrap();
        let mut theme = Theme::default();

        theme.added_bg = config.get_color("git-review.theme.added-bg").unwrap();

        assert_eq!(theme.added_bg, Color::Rgb(0x11, 0x22, 0x33));
    }

    #[test]
    fn parses_indexed_color() {
        assert_eq!(parse_color("123").unwrap(), Color::Indexed(123));
    }

    #[test]
    fn rejects_unknown_color() {
        assert!(parse_color("octarine").is_err());
    }
}
