//! One hover chip for the whole app: the frosted card every tooltip wears,
//! the gap that keeps it clear of the pointer, and the text builders most
//! call sites need.

use gpui::{AnyView, App, Div, IntoElement, SharedString, Window, div, prelude::*, px};

use crate::theme::Theme;

/// Distance the chip keeps from the pointer when it opens below-right.
const POINTER_GAP: f32 = 10.0;
/// Tighter distance once a window edge flips the chip to the other side, so
/// bottom and right-edge tooltips hug the pointer instead of floating away.
const EDGE_GAP: f32 = 4.0;
/// Vertical gap for [`text_tooltip_above`], which sits over the pointer.
const ABOVE_GAP: f32 = 6.0;

#[derive(Clone, Copy)]
pub(crate) enum Placement {
    Below,
    Above,
}

/// The one hover card: bordered, muted 11px text, frosted once by [`place`].
pub(crate) fn card(theme: &Theme) -> Div {
    div()
        .max_w(px(320.0))
        .px(px(9.0))
        .py(px(6.0))
        .rounded(px(6.0))
        .border_1()
        .border_color(theme.border)
        .bg(crate::popover::surface_bg(theme))
        .text_size(px(11.0))
        .text_color(theme.text_muted)
}

/// Frost a card and put it clear of the pointer: below-right by default, or
/// above for controls whose content the chip would cover.
pub(crate) fn place(card: impl IntoElement, placement: Placement) -> gpui::AnyElement {
    let card = crate::frost::frosted(6.0, crate::frost::MENU_BLUR, card);
    match placement {
        Placement::Below => div()
            .pt(px(POINTER_GAP))
            .pl(px(POINTER_GAP))
            .pb(px(EDGE_GAP))
            .pr(px(EDGE_GAP))
            .child(card)
            .into_any_element(),
        Placement::Above => div()
            .h(px(0.0))
            .flex()
            .flex_col()
            .justify_end()
            .child(div().relative().bottom(px(ABOVE_GAP)).child(card))
            .into_any_element(),
    }
}

pub struct TextTooltip {
    text: SharedString,
    placement: Placement,
}

impl gpui::Render for TextTooltip {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let card = card(Theme::of(cx)).child(self.text.clone());
        place(card, self.placement)
    }
}

/// `.tooltip(...)` builder for a one-line hover note.
pub fn text_tooltip(
    text: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text: SharedString = text.into();
    move |_, cx| {
        cx.new(|_| TextTooltip {
            text: text.clone(),
            placement: Placement::Below,
        })
        .into()
    }
}

/// Like [`text_tooltip`], shown above the pointer instead of below it, for
/// controls with content underneath that the chip would otherwise cover.
pub fn text_tooltip_above(
    text: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text: SharedString = text.into();
    move |_, cx| {
        cx.new(|_| TextTooltip {
            text: text.clone(),
            placement: Placement::Above,
        })
        .into()
    }
}
