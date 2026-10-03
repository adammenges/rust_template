//! Shared HEX-style visual vocabulary. See THIRD_PARTY.md for MIT attribution.
use gpui::{AnyElement, Div, IntoElement, SharedString, Stateful, div, prelude::*, px, rgb};
pub const CANVAS: u32 = 0x111111;
pub const SIDEBAR: u32 = 0x202020;
pub const SURFACE: u32 = 0x171717;
pub const SURFACE_HOVER: u32 = 0x1d1d1d;
pub const LINE: u32 = 0x292929;
pub const MUTED: u32 = 0x858585;
pub const TEXT: u32 = 0xeeeeee;
pub const TEXT_SOFT: u32 = 0xb8b8b8;
pub const ACCENT: u32 = 0x3b5cf6;
pub const NEGATIVE: u32 = 0xc98f89;
pub const TEXT_INPUT_HEIGHT: f32 = 34.;
pub const MULTILINE_INPUT_HEIGHT: f32 = 96.;
pub const CONTENT_WIDTH: f32 = 940.;

pub fn sidebar_width(viewport: f32) -> f32 {
    if viewport < 700. { 144. } else { 220. }
}
pub fn pane_header(title: &'static str, action: impl IntoElement, narrow: bool) -> Div {
    div()
        .h(px(70.))
        .px(px(if narrow { 16. } else { 32. }))
        .flex_none()
        .flex()
        .justify_center()
        .child(
            div()
                .w_full()
                .max_w(px(CONTENT_WIDTH))
                .h_full()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .border_b_1()
                .border_color(rgb(LINE))
                .child(
                    div()
                        .text_size(px(20.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(action),
        )
}
pub fn pane(description: &'static str, compact: bool) -> Div {
    div()
        .w_full()
        .max_w(px(CONTENT_WIDTH))
        .flex()
        .flex_col()
        .gap(px(if compact { 8. } else { 16. }))
        .child(hint(description))
}
pub fn hint(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(11.))
        .line_height(px(17.))
        .text_color(rgb(MUTED))
        .child(text.into())
}
pub fn card(compact: bool) -> Div {
    div()
        .w_full()
        .p(px(if compact { 12. } else { 20. }))
        .flex()
        .flex_col()
        .gap(px(if compact { 8. } else { 12. }))
        .rounded(px(10.))
        .border_1()
        .border_color(rgb(LINE))
        .bg(rgb(SURFACE))
}
pub fn section_label(label: &'static str) -> Div {
    div()
        .text_size(px(10.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(0x626262))
        .child(label)
}
pub fn button(
    id: &'static str,
    label: impl Into<SharedString>,
    shortcut: &'static str,
    primary: bool,
    disabled: bool,
) -> Stateful<Div> {
    let shortcut = shortcut.replace(
        "⌘/Ctrl ",
        if cfg!(target_os = "macos") {
            "⌘ "
        } else {
            "Ctrl "
        },
    );
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .px_3()
        .h(px(32.))
        .rounded(px(6.))
        .border_1()
        .border_color(rgb(if primary { 0x454f83 } else { LINE }))
        .bg(rgb(SURFACE))
        .text_size(px(12.))
        .text_color(rgb(if disabled { MUTED } else { TEXT_SOFT }))
        .when(!disabled, |b| {
            b.cursor_pointer().hover(move |b| {
                b.bg(rgb(if primary { 0x27315a } else { SURFACE_HOVER }))
                    .text_color(rgb(TEXT))
            })
        })
        .when(disabled, |b| b.opacity(0.5))
        .child(label.into())
        .child(hint(shortcut))
}
pub fn toggle(id: &'static str, enabled: bool, disabled: bool) -> Stateful<Div> {
    div()
        .id(id)
        .flex_none()
        .w(px(28.))
        .h(px(18.))
        .p(px(3.))
        .rounded(px(4.))
        .bg(rgb(if enabled { ACCENT } else { 0x3a3a3a }))
        .flex()
        .items_center()
        .when(enabled, |d| d.justify_end())
        .when(disabled, |d| d.opacity(0.5))
        .when(!disabled, |d| d.cursor_pointer().hover(|d| d.opacity(0.8)))
        .child(div().size(px(12.)).rounded(px(2.)).bg(rgb(TEXT)))
}
pub fn navigation(
    id: &'static str,
    label: &'static str,
    symbol: &'static str,
    selected: bool,
) -> Stateful<Div> {
    div()
        .id(id)
        .w_full()
        .h(px(38.))
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .rounded(px(6.))
        .cursor_pointer()
        .text_size(px(13.))
        .text_color(rgb(if selected { TEXT } else { MUTED }))
        .when(selected, |d| d.bg(rgb(0x3a3a3a)))
        .when(!selected, |d| {
            d.hover(|d| d.bg(rgb(0x2a2a2a)).text_color(rgb(TEXT_SOFT)))
        })
        .child(
            div()
                .size(px(22.))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(6.))
                .border_1()
                .border_color(rgb(if selected { 0x606060 } else { 0x3a3a3a }))
                .bg(rgb(if selected { 0x494949 } else { 0x2b2b2b }))
                .child(icon(symbol, selected)),
        )
        .child(label)
}
fn icon(symbol: &'static str, selected: bool) -> AnyElement {
    #[cfg(target_os = "macos")]
    {
        gpui_symbols::Icon::new(symbol)
            .size(px(12.))
            .color(rgb(if selected { TEXT } else { TEXT_SOFT }))
            .weight(gpui_symbols::SymbolWeight::Semibold)
            .rendering_mode(gpui_symbols::RenderingMode::Monochrome)
            .into_any_element()
    }
    #[cfg(target_os = "linux")]
    {
        div()
            .text_color(rgb(if selected { TEXT } else { TEXT_SOFT }))
            .child(if symbol == "house" { ">" } else { "*" })
            .into_any_element()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sidebar_leaves_room_for_content_at_minimum_width() {
        assert_eq!(sidebar_width(920.), 220.);
        assert!(480. - sidebar_width(480.) - 32. >= 300.);
    }
}
