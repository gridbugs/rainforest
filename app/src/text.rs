use crate::game_loop::CF;
use chargrid::{
    prelude::*,
    text::{StyledString, Text},
};

fn text_component(width: u32, text: Vec<StyledString>) -> CF<()> {
    Text::new(text)
        .wrap_word()
        .boxed_cf()
        .set_width(width)
        .press_any_key()
}

pub fn help(width: u32) -> CF<()> {
    let normal = Style::plain_text();
    let faint = Style::plain_text().with_foreground(Rgba32::new_grey(127));
    let f = |s: &str| StyledString {
        string: s.to_string(),
        style: faint,
    };
    let b = |s: &str| StyledString {
        string: s.to_string(),
        style: normal.with_bold(true),
    };
    let t = |s: &str| StyledString {
        string: s.to_string(),
        style: normal,
    };
    text_component(
        width,
        vec![
            b("Default Keyboard Controls\n"),
            t("Movement: Arrows/WASD/HJKL\n"),
            t("Wait: Space\n"),
            t("Examine: X\n"),
            t("\n"),
            b("Default Gamepad Controls\n"),
            t("Movement: D-Pad\n"),
            t("Wait: Select\n"),
            t("Examine: Right Bumper\n"),
            f("\n\nPress any key..."),
        ],
    )
}