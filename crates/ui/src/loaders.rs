//! Loaders: the space pulse loader, the gradient matrix spinner, and the boot
//! splash content. All motion routes through `crate::motion` pure helpers, so
//! the math is unit-tested and these elements are testable-by-compile.
//!
//! Rendering pattern: cells share a self-parking pulse clock; per-cell offsets
//! come from [`motion::staggered_phase`], so all cells stay phase-locked.
//! Cells animate inside fixed-size slots — opacity and inner size
//! are paint-local and never move surrounding layout. Reduced motion snaps every
//! cell to its rest state automatically (gpui `reduce_motion`). Activity grids
//! retain a gentle brightness pulse with system reduced motion; explicit On
//! and background pause keep them still.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EntityId, FillOptions, FillRule, IntoElement,
    ParentElement, PathBuilder, PathStyle, Render, RenderOnce, SharedString, Styled, Window,
    canvas, div, point, px,
};

use crate::icons::{self, icon};
use crate::motion::{self, GRADIENT_SPIN, SPLASH_OUT, SPACE_PULSE};
use crate::theme::{GlyphPalette, Theme};

// Shared with the terminal viewport (`space_proto::motion`) so both animate the
// same loaders from the same numbers.
pub use space_proto::motion::MATRIX_SIDE;

pub use space_proto::motion::{GSPIN_DIM, GSPIN_ROW_TINTS};

// ---------------------------------------------------------------------------
// Boot mark flipbook
// ---------------------------------------------------------------------------

// The boot splash's mark: the repository-root `space-logo-animation.svg`, a
// 2 s / 120-frame discrete SMIL morph (`attributeName="d"`). gpui does not run
// SMIL, so the flipbook is parsed out of the embedded file once and replayed
// by a canvas painter on a one-shot clock anchored at the splash's first
// paint, clamped to the final frame (the mark never loops). The painter fills
// with the theme foreground (the file's own white) so the face reads on both
// light and dark themes. Coordinates are authored in the 4096-unit viewBox;
// the path's own `scale(.1)` is applied while parsing, and the face's resting
// centre is the viewBox centre.
const SPACE_LOGO_SMIL: &[u8] = include_bytes!("../assets/space-logo-animation.svg");
const SPACE_LOGO_FRAMES: usize = 120;
const SPACE_LOGO_SPAN: Duration = Duration::from_secs(2);
const SPLASH_MARK_SIZE: f32 = 110.0;

/// One keyframe: closed subpaths in viewBox coordinates, centred on origin.
struct MarkFrame(Vec<Vec<(f32, f32)>>);

static MARK_FLIPBOOK: OnceLock<Option<Vec<MarkFrame>>> = OnceLock::new();
static MARK_EPOCH: OnceLock<Instant> = OnceLock::new();

fn parse_mark_flipbook() -> Option<Vec<MarkFrame>> {
    let text = std::str::from_utf8(SPACE_LOGO_SMIL).ok()?;
    // The file is one giant line: the flipbook lives in the animate tag's
    // values list, frames separated by `;`.
    let tag = text.split("<animate").nth(1)?;
    let values = tag.split("values=\"").nth(1)?;
    let values = values.split('"').next()?;
    let mut frames = Vec::with_capacity(SPACE_LOGO_FRAMES);
    for d in values.split(';') {
        frames.push(parse_mark_frame(d)?);
    }
    (frames.len() == SPACE_LOGO_FRAMES).then_some(frames)
}

/// Walk one frame's `d`: `M x y` opens a subpath, `l` chains relative integer
/// pairs, `z` closes. Whitespace and signs are the only separators (the
/// generator emits no decimals or commas); anything unexpected fails the
/// parse, so the splash falls back to the static icon.
fn parse_mark_frame(d: &str) -> Option<MarkFrame> {
    let bytes = d.as_bytes();
    let mut subpaths: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'M' => {
                let (x, next) = read_mark_int(bytes, i + 1)?;
                let (y, next) = read_mark_int(bytes, next)?;
                i = next;
                subpaths.push(vec![mark_point(x, y)]);
            }
            b'l' => {
                i += 1;
                let sub = subpaths.last_mut()?;
                let (mut x, mut y) = *sub.last()?;
                loop {
                    if !starts_mark_int(bytes, i) {
                        break;
                    }
                    let (dx, next) = read_mark_int(bytes, i)?;
                    let (dy, next) = read_mark_int(bytes, next)?;
                    i = next;
                    x += dx * 0.1;
                    y += dy * 0.1;
                    sub.push((x, y));
                }
            }
            b'z' | b'Z' => i += 1,
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            _ => return None,
        }
    }
    (!subpaths.is_empty()).then_some(MarkFrame(subpaths))
}

fn starts_mark_int(bytes: &[u8], mut i: usize) -> bool {
    while matches!(bytes.get(i), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        i += 1;
    }
    matches!(bytes.get(i), Some(b'-') | Some(b'0'..=b'9'))
}

fn read_mark_int(bytes: &[u8], mut i: usize) -> Option<(f32, usize)> {
    while matches!(bytes.get(i), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        i += 1;
    }
    let negative = matches!(bytes.get(i), Some(b'-'));
    if negative {
        i += 1;
    }
    let first = i;
    let mut value: i64 = 0;
    while matches!(bytes.get(i), Some(b'0'..=b'9')) {
        value = value * 10 + i64::from(bytes[i] - b'0');
        i += 1;
    }
    (i > first).then(|| (if negative { -(value as f32) } else { value as f32 }, i))
}

/// `scale(.1)` maps the authored integers into the 4096-unit viewBox; shift
/// by the viewBox centre so the spin pivots in place.
fn mark_point(x: f32, y: f32) -> (f32, f32) {
    (x * 0.1 - 2048.0, y * 0.1 - 2048.0)
}

/// The one-shot clock: anchored at the splash's first paint.
fn mark_epoch() -> Instant {
    *MARK_EPOCH.get_or_init(Instant::now)
}

/// Whether the boot mark is mid-morph and needs frames: the flipbook parsed,
/// motion is allowed, and the one-shot clock is still inside its span.
pub fn boot_mark_playing(cx: &App) -> bool {
    !cx.reduce_motion()
        && MARK_FLIPBOOK.get_or_init(parse_mark_flipbook).is_some()
        && mark_epoch().elapsed() < SPACE_LOGO_SPAN
}

/// The boot splash logo: the Space face's one-shot spin, sized to a square
/// slot of `height_px` (the face spans four fifths of it). Under reduced
/// motion it rests on the full-face end frame; a parse failure falls back to
/// the static logo icon.
pub fn space_logo_animation(theme: &Theme, height_px: f32, cx: &App) -> AnyElement {
    // The theme's foreground ink is already the contrast-positive side of the
    // background: bright on dark, dark on light. Painting it solid gives the
    // adaptive overlay read without an alpha wash over the mark.
    let color = theme.text;
    let flipbook = MARK_FLIPBOOK.get_or_init(parse_mark_flipbook);
    // The frame index rides the one-shot clock; the last frame holds.
    let frame = match flipbook {
        Some(frames) if !cx.reduce_motion() => {
            let step = (mark_epoch().elapsed().as_secs_f32() / SPACE_LOGO_SPAN.as_secs_f32()
                * SPACE_LOGO_FRAMES as f32) as usize;
            Some(&frames[step.min(SPACE_LOGO_FRAMES - 1)])
        }
        Some(frames) => Some(&frames[SPACE_LOGO_FRAMES - 1]),
        None => None,
    };
    match frame {
        Some(frame) => {
            let MarkFrame(subpaths) = frame;
            div()
                .relative()
                .size(px(height_px))
                .child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, _, window, _| {
                            // Paint-local: the mark spins inside a fixed square
                            // slot, mapped from the 4096-unit viewBox.
                            let side =
                                f32::from(bounds.size.width).min(f32::from(bounds.size.height));
                            let scale = side / 4096.0;
                            let center = bounds.center();
                            let mut builder = PathBuilder::default().with_style(PathStyle::Fill(
                                FillOptions::default().with_fill_rule(FillRule::EvenOdd),
                            ));
                            for subpath in subpaths {
                                for (index, &(x, y)) in subpath.iter().enumerate() {
                                    let at = point(
                                        center.x + px(x * scale),
                                        center.y + px(y * scale),
                                    );
                                    if index == 0 {
                                        builder.move_to(at);
                                    } else {
                                        builder.line_to(at);
                                    }
                                }
                                builder.close();
                            }
                            if let Ok(path) = builder.build() {
                                window.paint_path(path, color);
                            }
                        },
                    )
                    // The canvas only paints what it is given bounds for;
                    // fill the square slot.
                    .absolute()
                    .inset_0(),
                )
                .into_any_element()
        }
        None => icon(icons::SPACE_LOGO)
            .size(px(height_px))
            .text_color(color)
            .into_any_element(),
    }
}

/// The gradient matrix spinner (WorkingIndicator), ported from space's
/// gradient-spin.tsx: a 3×3 grid of round cells tinted per row from the
/// sunrise gradient. Each cell pulses opacity once per 750ms period; the
/// per-cell phase follows the "arrow-up" pattern (the pulse enters at the
/// bottom edge and converges toward the top-center cell), so the wave reads
/// as travelling upward.
pub fn gradient_spinner(
    _id: &'static str,
    _theme: &Theme,
    cell_px: f32,
    view: EntityId,
    cx: &mut App,
) -> impl IntoElement {
    let center = (MATRIX_SIDE as f32 - 1.0) / 2.0;
    let max = MATRIX_SIDE as f32 - 1.0 + center;
    let pulse = motion::activity_pulse_slow(view, cx);
    div()
        .flex()
        .flex_col()
        .gap(px(cell_px / 2.0))
        .children((0..MATRIX_SIDE).map(move |row| {
            let tint: gpui::Hsla = gpui::rgb(GSPIN_ROW_TINTS[row]).into();
            div()
                .flex()
                .flex_row()
                .gap(px(cell_px / 2.0))
                .children((0..MATRIX_SIDE).map(move |col| {
                    // Distance of this cell from the wave origin, normalized
                    // into a phase offset (gradient-spin's `--gspin-phase`).
                    let d = MATRIX_SIDE as f32 - 1.0 - row as f32 + (col as f32 - center).abs();
                    let phase = if max == 0.0 { 0.0 } else { d / (max + 1.0) };
                    div()
                        .size(px(cell_px))
                        .rounded(px(cell_px / 2.0))
                        .bg(tint)
                        .opacity(pulse.opacity(phase, GSPIN_DIM))
                }))
        }))
}

/// A 2×3 activity glyph sized for compact status slots. Its color is an
/// explicit accent-preset role supplied by the caller, while brightness snakes
/// around the grid's perimeter as a tiny radial chase.
pub fn mini_glyph_spinner(
    key: impl Into<SharedString>,
    cell_px: f32,
    palette: GlyphPalette,
    view: EntityId,
    cx: &mut App,
) -> impl IntoElement {
    mini_spinner_tinted(key, cell_px, palette.rows(), view, cx)
}

/// Grayscale variant for surfaces where an accent would pull focus (the
/// sidebar connection line): same grid, snake, and timing, color left to the
/// caller.
pub fn mini_mono_spinner(
    key: impl Into<SharedString>,
    cell_px: f32,
    tint: gpui::Hsla,
    view: EntityId,
    cx: &mut App,
) -> impl IntoElement {
    mini_spinner_tinted(key, cell_px, [tint; 3], view, cx)
}

fn mini_spinner_tinted(
    key: impl Into<SharedString>,
    cell_px: f32,
    row_tints: [gpui::Hsla; 3],
    _view: EntityId,
    _cx: &mut App,
) -> impl IntoElement {
    MiniSpinner {
        key: key.into(),
        cell_px,
        row_tints,
    }
}

#[derive(IntoElement)]
struct MiniSpinner {
    key: SharedString,
    cell_px: f32,
    row_tints: [gpui::Hsla; 3],
}

impl RenderOnce for MiniSpinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Keep pulse invalidation separate from container state changes so
        // cached sibling rows can be reused while these six cells animate.
        let view = window.with_global_id(self.key.into(), |id, window| {
            window.with_element_state(id, |previous: Option<Entity<MiniSpinnerView>>, _| {
                let view = previous.unwrap_or_else(|| {
                    cx.new(|_| MiniSpinnerView {
                        cell_px: self.cell_px,
                        row_tints: self.row_tints,
                    })
                });
                view.update(cx, |view, cx| {
                    if view.cell_px != self.cell_px || view.row_tints != self.row_tints {
                        view.cell_px = self.cell_px;
                        view.row_tints = self.row_tints;
                        cx.notify();
                    }
                });
                (view.clone(), view)
            })
        });
        view.cached(
            gpui::StyleRefinement::default()
                .w(px(self.cell_px * 2.5))
                .h(px(self.cell_px * 4.0)),
        )
    }
}

struct MiniSpinnerView {
    cell_px: f32,
    row_tints: [gpui::Hsla; 3],
}

impl Render for MiniSpinnerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        mini_spinner_cells(self.cell_px, self.row_tints, cx.entity_id(), cx)
    }
}

fn mini_spinner_cells(
    cell_px: f32,
    row_tints: [gpui::Hsla; 3],
    view: EntityId,
    cx: &mut App,
) -> impl IntoElement {
    const COLS: usize = 2;
    const ROWS: usize = 3;
    /// Clockwise ring position of each `(row, col)` cell, top-left first:
    /// (0,0) → (0,1) → (1,1) → (2,1) → (2,0) → (1,0).
    const RING: [[usize; COLS]; ROWS] = [[0, 1], [5, 2], [4, 3]];
    const RING_LEN: f32 = (COLS * ROWS) as f32;
    let pulse = motion::activity_pulse(view, cx);
    div()
        .flex()
        .flex_col()
        .gap(px(cell_px / 2.0))
        .children((0..ROWS).map(move |row| {
            let tint = row_tints[row];
            div()
                .flex()
                .flex_row()
                .gap(px(cell_px / 2.0))
                .children((0..COLS).map(move |col| {
                    let phase = RING[row][col] as f32 / RING_LEN;
                    div()
                        .size(px(cell_px))
                        .rounded(px(cell_px / 2.0))
                        .bg(tint)
                        .opacity(pulse.opacity(phase, GSPIN_DIM))
                }))
        }))
}

/// A rounded fill that breathes on the space pulse clock between `floor` and
/// `peak` opacity, filling whatever positioned box it is placed in. Like [`pulse_dot`] it repaints from its own cached
/// view, so the surrounding chrome stays still.
pub fn pulse_glow(
    key: impl Into<SharedString>,
    radius: f32,
    tint: gpui::Hsla,
    floor: f32,
    peak: f32,
) -> impl IntoElement {
    PulseGlow {
        key: key.into(),
        style: GlowStyle {
            radius,
            tint,
            floor,
            peak,
        },
    }
}

#[derive(Clone, Copy, PartialEq)]
struct GlowStyle {
    radius: f32,
    tint: gpui::Hsla,
    floor: f32,
    peak: f32,
}

#[derive(IntoElement)]
struct PulseGlow {
    key: SharedString,
    style: GlowStyle,
}

impl RenderOnce for PulseGlow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.style;
        let view = window.with_global_id(self.key.into(), |id, window| {
            window.with_element_state(id, |previous: Option<Entity<PulseGlowView>>, _| {
                let view = previous.unwrap_or_else(|| cx.new(|_| PulseGlowView { style }));
                view.update(cx, |view, cx| {
                    if view.style != style {
                        view.style = style;
                        cx.notify();
                    }
                });
                (view.clone(), view)
            })
        });
        view.cached(gpui::StyleRefinement::default().size_full())
    }
}

struct PulseGlowView {
    style: GlowStyle,
}

impl Render for PulseGlowView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let GlowStyle {
            radius,
            tint,
            floor,
            peak,
        } = self.style;
        let delta = motion::pulse_delta_slow(&SPACE_PULSE, cx.entity_id(), cx);
        // pulse_opacity spans 0.08..1; renormalise to 0..1 before mapping.
        let wave = (motion::pulse_opacity(delta) - 0.08) / 0.92;
        div()
            .size_full()
            .rounded(px(radius))
            .bg(tint)
            .opacity(motion::lerp(floor, peak, wave))
    }
}

/// Dashes around a [`ring_spinner`].
const SPIN_DASHES: usize = 12;
/// One full turn of a [`ring_spinner`].
const SPIN_PERIOD: std::time::Duration = std::time::Duration::from_millis(900);

/// The loading spinner for everything that isn't a session at work (those
/// keep [`mini_glyph_spinner`] and [`gradient_spinner`]): a ring of short
/// dashes, brightest at the head and fading round to a gap at the tail,
/// turning clockwise. Only the dashes paint, so it sits on glass, washes and
/// images alike. It repaints from its own cached view, so the chrome around
/// it stays still; with reduced motion it rests.
pub fn ring_spinner(
    key: impl Into<SharedString>,
    diameter: f32,
    tint: gpui::Hsla,
) -> impl IntoElement {
    RingSpinner {
        key: key.into(),
        diameter,
        tint,
    }
}

#[derive(IntoElement)]
struct RingSpinner {
    key: SharedString,
    diameter: f32,
    tint: gpui::Hsla,
}

impl RenderOnce for RingSpinner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (diameter, tint) = (self.diameter, self.tint);
        let view = window.with_global_id(self.key.into(), |id, window| {
            window.with_element_state(id, |previous: Option<Entity<RingSpinnerView>>, _| {
                let view =
                    previous.unwrap_or_else(|| cx.new(|_| RingSpinnerView { diameter, tint }));
                view.update(cx, |view, cx| {
                    if view.diameter != diameter || view.tint != tint {
                        view.diameter = diameter;
                        view.tint = tint;
                        cx.notify();
                    }
                });
                (view.clone(), view)
            })
        });
        view.cached(gpui::StyleRefinement::default().size(px(diameter)))
    }
}

struct RingSpinnerView {
    diameter: f32,
    tint: gpui::Hsla,
}

impl Render for RingSpinnerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let phase = motion::spin_phase(cx.entity_id(), SPIN_PERIOD, cx).unwrap_or(0.0);
        let (diameter, tint) = (self.diameter, self.tint);
        div().size(px(diameter)).child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let stroke = (diameter * 0.11).max(1.5);
                    let center = bounds.center();
                    let radius = diameter / 2.0 - stroke;
                    let slot = std::f32::consts::TAU / SPIN_DASHES as f32;
                    // The head leads at 12 o'clock plus the turn; each dash
                    // behind it is dimmer, the last few nearly gone.
                    let head = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * phase;
                    for age in 0..SPIN_DASHES {
                        let opacity = (1.0 - age as f32 / SPIN_DASHES as f32).powf(1.6);
                        if opacity < 0.04 {
                            continue;
                        }
                        let start = head - slot * age as f32 - slot * 0.62;
                        let end = head - slot * age as f32;
                        let at = |theta: f32| {
                            point(
                                center.x + px(radius * theta.cos()),
                                center.y + px(radius * theta.sin()),
                            )
                        };
                        let mut builder = PathBuilder::stroke(px(stroke));
                        builder.move_to(at(start));
                        for step in 1..=4 {
                            builder.line_to(at(start + (end - start) * step as f32 / 4.0));
                        }
                        if let Ok(path) = builder.build() {
                            window.paint_path(path, tint.opacity(opacity));
                        }
                    }
                },
            )
            .size_full(),
        )
    }
}

/// Stroke width of [`upload_progress_ring`].
const RING_STROKE: f32 = 2.5;
/// Polyline segments for a full circle — plenty for a ≤40px ring.
const RING_SEGMENTS: f32 = 64.0;

/// Radial upload-progress ring with the percent centered — overlaid on a
/// sending echo's attachment thumbnail while its bytes cross the relay
/// (2026-08-18 "Sending… forever" report; the thumbnail is where the wait
/// visibly belongs). A faint full track plus a bright arc growing clockwise
/// from 12 o'clock; gpui paths have no arc primitive, so both are stroked
/// polylines. Fixed white-on-wash palette: the caller dims the image behind
/// it, which reads in both themes.
pub fn upload_progress_ring(percent: u8, diameter: f32) -> AnyElement {
    let ring = progress_arc(
        percent,
        diameter,
        gpui::hsla(0.0, 0.0, 1.0, 0.22),
        gpui::hsla(0.0, 0.0, 1.0, 0.95),
    );
    div()
        .relative()
        .size(px(diameter))
        .flex()
        .items_center()
        .justify_center()
        .child(ring)
        .child(
            div()
                .text_size(px(9.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(gpui::hsla(0.0, 0.0, 1.0, 0.95))
                .child(SharedString::from(format!("{percent}%"))),
        )
        .into_any_element()
}

/// [`upload_progress_ring`] without the percent, in the caller's colors: for
/// a spot too small to label, such as a sent chip's icon well.
pub fn upload_progress_arc(
    percent: u8,
    diameter: f32,
    track: gpui::Hsla,
    arc: gpui::Hsla,
) -> AnyElement {
    div()
        .relative()
        .size(px(diameter))
        .child(progress_arc(percent, diameter, track, arc))
        .into_any_element()
}

/// A faint full track plus an arc growing clockwise from 12 o'clock, filling
/// its parent.
fn progress_arc(
    percent: u8,
    diameter: f32,
    track: gpui::Hsla,
    arc: gpui::Hsla,
) -> gpui::Canvas<()> {
    let frac = f32::from(percent.min(100)) / 100.0;
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let center = bounds.center();
            let radius = diameter / 2.0 - RING_STROKE;
            let mut paint_arc = |sweep: f32, color: gpui::Hsla| {
                if sweep <= 0.0 {
                    return;
                }
                let steps = ((RING_SEGMENTS * sweep).ceil() as usize).max(2);
                let at = |i: usize| {
                    // Clockwise from 12 o'clock.
                    let theta = -std::f32::consts::FRAC_PI_2
                        + std::f32::consts::TAU * sweep * (i as f32 / steps as f32);
                    point(
                        center.x + px(radius * theta.cos()),
                        center.y + px(radius * theta.sin()),
                    )
                };
                let mut builder = PathBuilder::stroke(px(RING_STROKE));
                builder.move_to(at(0));
                for i in 1..=steps {
                    builder.line_to(at(i));
                }
                if let Ok(path) = builder.build() {
                    window.paint_path(path, color);
                }
            };
            paint_arc(1.0, track);
            paint_arc(frac, arc);
        },
    )
    .absolute()
    .inset_0()
}

/// Full-window boot splash: the Space mark's one-shot spin (it plays once and
/// rests on the full-face frame) centred on the page tone. While `fading` it
/// plays `splash-out` (150ms hold, then 0.5s fade); the shell removes it once
/// [`SPLASH_OUT`] has run its course.
pub fn splash_overlay(theme: &Theme, fading: bool, cx: &App) -> AnyElement {
    let content = div()
        .absolute()
        .inset_0()
        // The boot overlay fills with the page tone: one fill, one entrance.
        // Nothing mounts behind it mid-fade, so the reveal stays a single move.
        .bg(theme.bg)
        .flex()
        .items_center()
        .justify_center()
        .child(space_logo_animation(theme, SPLASH_MARK_SIZE, cx));
    if fading {
        motion::splash_out("boot-splash-out", content).into_any_element()
    } else {
        content.into_any_element()
    }
}
// Compile-time proof the specs referenced here stay wired to the catalog.
const _: () = {
    assert!(SPLASH_OUT.delay_ms == 150);
    assert!(SPACE_PULSE.duration_ms == 2400);
    assert!(GRADIENT_SPIN.duration_ms == 750);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_mark_flipbook_is_complete() {
        let frames = parse_mark_flipbook().expect("embedded SMIL flipbook must parse");
        assert_eq!(frames.len(), SPACE_LOGO_FRAMES);
        // The morph opens edge-on (a single sliver) and settles as the full
        // eight-part face.
        assert!(frames[0].0.len() < frames[SPACE_LOGO_FRAMES - 1].0.len());
        for MarkFrame(subpaths) in &frames {
            assert!(!subpaths.is_empty(), "every frame has at least one subpath");
            for subpath in subpaths {
                assert!(subpath.len() >= 3, "closed subpaths need a face at least");
            }
        }
    }

    #[test]
    fn boot_mark_settles_centred_on_the_viewbox() {
        let frames = parse_mark_flipbook().expect("embedded SMIL flipbook must parse");
        let MarkFrame(subpaths) = &frames[SPACE_LOGO_FRAMES - 1];
        let points = subpaths.iter().flatten();
        let (min_x, max_x) = points
            .clone()
            .map(|p| p.0)
            .fold((f32::MAX, f32::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)));
        let (min_y, max_y) = points
            .map(|p| p.1)
            .fold((f32::MAX, f32::MIN), |(lo, hi), y| (lo.min(y), hi.max(y)));
        // The face fills four fifths of the viewBox, centred on its middle —
        // the spin pivots in place.
        assert!((max_x - min_x - 3278.0).abs() < 4.0);
        assert!((max_y - min_y - 3278.0).abs() < 4.0);
        assert!((max_x + min_x).abs() < 4.0);
        assert!((max_y + min_y).abs() < 4.0);
    }

    #[test]
    fn boot_mark_flipbook_tessellates_every_frame() {
        // The painter swallows `build()` failures (`if let Ok(path)`), so a
        // frame that failed to tessellate would silently draw nothing while
        // the splash still looks "gone". Prove every frame builds a
        // non-degenerate path.
        let frames = parse_mark_flipbook().expect("embedded SMIL flipbook must parse");
        for (i, MarkFrame(subpaths)) in frames.iter().enumerate() {
            let mut builder = PathBuilder::default().with_style(PathStyle::Fill(
                FillOptions::default().with_fill_rule(FillRule::EvenOdd),
            ));
            for subpath in subpaths {
                for (index, &(x, y)) in subpath.iter().enumerate() {
                    let at = point(px(x), px(y));
                    if index == 0 {
                        builder.move_to(at);
                    } else {
                        builder.line_to(at);
                    }
                }
                builder.close();
            }
            let path = builder
                .build()
                .unwrap_or_else(|err| panic!("frame {i} must tessellate: {err:?}"));
            assert!(
                path.bounds.size.width > px(0.0) && path.bounds.size.height > px(0.0),
                "frame {i} tessellated to an empty path"
            );
        }
    }
}
