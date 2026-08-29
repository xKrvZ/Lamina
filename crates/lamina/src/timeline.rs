//! Domain-neutral timeline / dopesheet / curve-graph widgets.

use crate::context::GuiContext;
use crate::id::Id;
use crate::style::{self, FONT_SCALE, TYPE_CAPTION, TYPE_LABEL};
use crate::types::{Color, Rect};

#[derive(Debug, Clone, Copy)]
pub struct TimeView {
    pub t0: f32,
    pub t1: f32,
}

impl TimeView {
    pub fn time_to_x(self, t: f32, lane: Rect) -> f32 {
        let span = (self.t1 - self.t0).max(1e-4);
        lane.min_x + (t - self.t0) / span * lane.width()
    }

    pub fn x_to_time(self, x: f32, lane: Rect) -> f32 {
        let span = (self.t1 - self.t0).max(1e-4);
        let u = ((x - lane.min_x) / lane.width()).clamp(0.0, 1.0);
        self.t0 + u * span
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimelineKey {
    pub time: f32,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TimelineEvent {
    SetPlayhead(f32),
    SelectKey { track: usize, key: usize },
    DragKey { track: usize, key: usize, time: f32 },
}

/// Time ruler. Click / drag sets playhead. Returns new time while dragging or on click.
pub fn timeline_ruler(
    ui: &mut GuiContext<'_>,
    id: Id,
    rect: Rect,
    view: TimeView,
    playhead: f32,
) -> Option<f32> {
    ui.panel(rect, style::DOCK_BG);
    ui.panel(
        Rect::from_pos_size(rect.min_x, rect.max_y - 1.0, rect.width(), 1.0),
        style::SEPARATOR,
    );

    let span = (view.t1 - view.t0).max(1e-4);
    let mut step = 0.5;
    let px_per_sec = rect.width() / span;
    if px_per_sec > 80.0 {
        step = 0.1;
    } else if px_per_sec < 20.0 {
        step = 1.0;
    }
    let mut t = (view.t0 / step).floor() * step;
    while t <= view.t1 + 1e-4 {
        let x = view.time_to_x(t, rect);
        if x >= rect.min_x && x <= rect.max_x {
            ui.panel(
                Rect::from_pos_size(x, rect.max_y - 8.0, 1.0, 8.0),
                style::TEXT_MUTED,
            );
            ui.label_at(
                x + 3.0,
                rect.min_y + 4.0,
                &format!("{t:.1}"),
                style::TEXT_MUTED,
                FONT_SCALE * TYPE_CAPTION,
            );
        }
        t += step;
    }

    let hovered = ui.pointer_in(rect);
    if hovered {
        ui.state.set_hot(id);
    }
    if hovered && ui.input.primary_pressed {
        ui.state.active = Some(id);
    }
    let dragging = ui.state.is_active(id) && ui.input.primary_down;
    if dragging || (hovered && ui.input.primary_pressed) {
        if let Some((x, _)) = ui.input.pointer {
            return Some(view.x_to_time(x, rect).clamp(view.t0, view.t1));
        }
    }
    let _ = playhead;
    None
}

/// Vertical playhead in a lane. Drag to scrub. Returns new time while dragging.
pub fn playhead(
    ui: &mut GuiContext<'_>,
    id: Id,
    lane: Rect,
    view: TimeView,
    time: f32,
) -> Option<f32> {
    let x = view.time_to_x(time, lane).clamp(lane.min_x, lane.max_x);
    let hit = Rect::from_pos_size(x - 4.0, lane.min_y, 8.0, lane.height());
    let hovered = ui.pointer_in(hit) || ui.pointer_in(lane);
    if hovered {
        ui.state.set_hot(id);
    }
    if ui.pointer_in(hit) && ui.input.primary_pressed {
        ui.state.active = Some(id);
    }
    ui.panel(
        Rect::from_pos_size(x, lane.min_y, 2.0, lane.height()),
        style::ACCENT,
    );
    ui.panel_rounded(
        Rect::from_pos_size(x - 4.0, lane.min_y, 10.0, 8.0),
        style::ACCENT,
        2.0,
    );
    if ui.state.is_active(id) && ui.input.primary_down {
        if let Some((px, _)) = ui.input.pointer {
            return Some(view.x_to_time(px, lane).clamp(view.t0, view.t1));
        }
    }
    None
}

/// One dopesheet row. Returns a timeline event.
pub fn track_row(
    ui: &mut GuiContext<'_>,
    id: Id,
    rect: Rect,
    label: &str,
    view: TimeView,
    keys: &[TimelineKey],
    track_index: usize,
    selected: bool,
) -> Option<TimelineEvent> {
    ui.panel(
        rect,
        if selected {
            style::SELECTED_BG
        } else {
            style::TRACK_BG
        },
    );
    ui.label_at(
        rect.min_x + 6.0,
        rect.min_y + 6.0,
        label,
        style::TEXT_DIM,
        FONT_SCALE * TYPE_LABEL,
    );
    let lane = Rect::from_min_max(rect.min_x + 96.0, rect.min_y, rect.max_x, rect.max_y);
    ui.panel(lane, style::SURFACE);
    let mut event = None;
    for (ki, key) in keys.iter().enumerate() {
        let ev = key_marker(
            ui,
            id.child("key").with(ki as u64),
            lane,
            view,
            key.time,
            key.selected,
        );
        if let Some(t) = ev {
            event = Some(if ui.input.primary_down && ui.state.is_active(id.child("key").with(ki as u64)) {
                TimelineEvent::DragKey {
                    track: track_index,
                    key: ki,
                    time: t,
                }
            } else {
                TimelineEvent::SelectKey {
                    track: track_index,
                    key: ki,
                }
            });
        }
    }
    event
}

/// Diamond key on a time lane. Click selects; drag returns new time.
pub fn key_marker(
    ui: &mut GuiContext<'_>,
    id: Id,
    lane: Rect,
    view: TimeView,
    time: f32,
    selected: bool,
) -> Option<f32> {
    let x = view.time_to_x(time, lane);
    let s = 7.0;
    let hit = Rect::from_pos_size(x - s, lane.min_y + (lane.height() - s * 2.0) * 0.5, s * 2.0, s * 2.0);
    let hovered = ui.pointer_in(hit);
    if hovered {
        ui.state.set_hot(id);
    }
    if hovered && ui.input.primary_pressed {
        ui.state.active = Some(id);
    }
    let color = if selected {
        style::ACCENT
    } else if hovered {
        style::TEXT
    } else {
        style::TEXT_DIM
    };
    ui.panel_rounded(hit, color, 2.0);
    if ui.state.is_active(id) && ui.input.primary_down {
        if let Some((px, _)) = ui.input.pointer {
            return Some(view.x_to_time(px, lane));
        }
    }
    if ui.input.primary_released && ui.state.is_active(id) && hovered {
        return Some(time);
    }
    None
}

#[derive(Debug, Clone, Copy)]
pub struct CurveKeyVis {
    pub time: f32,
    pub value: f32,
    pub selected: bool,
    pub in_dt: f32,
    pub in_dv: f32,
    pub out_dt: f32,
    pub out_dv: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum CurveEvent {
    Select(usize),
    MoveKey { index: usize, time: f32, value: f32 },
    MoveHandle {
        index: usize,
        incoming: bool,
        dt: f32,
        dv: f32,
    },
}

#[derive(Clone, Copy)]
pub struct ValueView {
    pub v0: f32,
    pub v1: f32,
}

impl ValueView {
    fn to_y(self, v: f32, plot: Rect) -> f32 {
        let span = (self.v1 - self.v0).max(1e-4);
        let u = (v - self.v0) / span;
        plot.max_y - u * plot.height()
    }

    fn from_y(self, y: f32, plot: Rect) -> f32 {
        let u = ((plot.max_y - y) / plot.height()).clamp(0.0, 1.0);
        self.v0 + u * (self.v1 - self.v0)
    }
}

/// Plot a value curve. `sample` is called for many times to stroke the path.
pub fn curve_graph(
    ui: &mut GuiContext<'_>,
    id: Id,
    rect: Rect,
    view: TimeView,
    values: ValueView,
    keys: &[CurveKeyVis],
    playhead: f32,
    mut sample: impl FnMut(f32) -> f32,
) -> Option<CurveEvent> {
    ui.panel(rect, style::TRACK_BG);
    let plot = Rect::from_min_max(rect.min_x + 8.0, rect.min_y + 8.0, rect.max_x - 8.0, rect.max_y - 8.0);
    ui.panel(plot, style::SURFACE);

    let mut prev: Option<(f32, f32)> = None;
    let steps = 64.max(plot.width() as i32 / 4) as usize;
    for i in 0..=steps {
        let u = i as f32 / steps as f32;
        let t = view.t0 + u * (view.t1 - view.t0);
        let v = sample(t);
        let x = view.time_to_x(t, plot);
        let y = values.to_y(v, plot);
        if let Some((px, py)) = prev {
            let minx = px.min(x);
            let miny = py.min(y);
            ui.panel(
                Rect::from_pos_size(minx, miny, (px - x).abs().max(1.0), (py - y).abs().max(1.0)),
                style::ACCENT_DIM,
            );
        }
        prev = Some((x, y));
    }

    let px = view.time_to_x(playhead, plot);
    ui.panel(
        Rect::from_pos_size(px, plot.min_y, 1.0, plot.height()),
        style::ACCENT,
    );

    let mut event = None;
    for (i, key) in keys.iter().enumerate() {
        let x = view.time_to_x(key.time, plot);
        let y = values.to_y(key.value, plot);
        if key.selected {
            let hx0 = view.time_to_x(key.time + key.in_dt, plot);
            let hy0 = values.to_y(key.value + key.in_dv, plot);
            let hx1 = view.time_to_x(key.time + key.out_dt, plot);
            let hy1 = values.to_y(key.value + key.out_dv, plot);
            draw_handle(ui, id.child("hin").with(i as u64), hx0, hy0, x, y);
            draw_handle(ui, id.child("hout").with(i as u64), hx1, hy1, x, y);
            if let Some(ev) = drag_handle(
                ui,
                id.child("hin").with(i as u64),
                hx0,
                hy0,
                true,
                i,
                key,
                view,
                values,
                plot,
            ) {
                event = Some(ev);
            }
            if let Some(ev) = drag_handle(
                ui,
                id.child("hout").with(i as u64),
                hx1,
                hy1,
                false,
                i,
                key,
                view,
                values,
                plot,
            ) {
                event = Some(ev);
            }
        }
        let hit = Rect::from_pos_size(x - 5.0, y - 5.0, 10.0, 10.0);
        let hovered = ui.pointer_in(hit);
        if hovered {
            ui.state.set_hot(id.child("k").with(i as u64));
        }
        if hovered && ui.input.primary_pressed {
            ui.state.active = Some(id.child("k").with(i as u64));
            event = Some(CurveEvent::Select(i));
        }
        ui.panel_rounded(
            hit,
            if key.selected {
                style::ACCENT
            } else if hovered {
                style::TEXT
            } else {
                style::TEXT_DIM
            },
            2.0,
        );
        if ui.state.is_active(id.child("k").with(i as u64)) && ui.input.primary_down {
            if let Some((px, py)) = ui.input.pointer {
                event = Some(CurveEvent::MoveKey {
                    index: i,
                    time: view.x_to_time(px, plot),
                    value: values.from_y(py, plot),
                });
            }
        }
    }
    event
}

fn draw_handle(ui: &mut GuiContext<'_>, _id: Id, hx: f32, hy: f32, kx: f32, ky: f32) {
    ui.panel(
        Rect::from_pos_size(
            hx.min(kx),
            hy.min(ky),
            (hx - kx).abs().max(1.0),
            (hy - ky).abs().max(1.0),
        ),
        style::TEXT_MUTED,
    );
    ui.panel_rounded(Rect::from_pos_size(hx - 4.0, hy - 4.0, 8.0, 8.0), style::WARNING, 4.0);
}

fn drag_handle(
    ui: &mut GuiContext<'_>,
    id: Id,
    hx: f32,
    hy: f32,
    incoming: bool,
    index: usize,
    key: &CurveKeyVis,
    view: TimeView,
    values: ValueView,
    plot: Rect,
) -> Option<CurveEvent> {
    let hit = Rect::from_pos_size(hx - 5.0, hy - 5.0, 10.0, 10.0);
    if ui.pointer_in(hit) {
        ui.state.set_hot(id);
    }
    if ui.pointer_in(hit) && ui.input.primary_pressed {
        ui.state.active = Some(id);
    }
    if ui.state.is_active(id) && ui.input.primary_down {
        if let Some((px, py)) = ui.input.pointer {
            let t = view.x_to_time(px, plot);
            let v = values.from_y(py, plot);
            return Some(CurveEvent::MoveHandle {
                index,
                incoming,
                dt: t - key.time,
                dv: v - key.value,
            });
        }
    }
    None
}

/// Record control: idle / recording. Returns true on click.
pub fn record_button(ui: &mut GuiContext<'_>, id: Id, rect: Rect, recording: bool) -> bool {
    let hovered = ui.pointer_in(rect);
    if hovered {
        ui.state.set_hot(id);
    }
    if hovered && ui.input.primary_pressed {
        ui.state.active = Some(id);
    }
    let clicked = ui.input.primary_released && ui.state.is_active(id) && hovered;
    let bg = if recording {
        style::ERROR
    } else if hovered {
        style::BUTTON_HOVER
    } else {
        style::BUTTON_BG
    };
    ui.panel_rounded(rect, bg, style::RADIUS_PILL);
    let label = if recording { "Stop" } else { "Record" };
    ui.label_centered_in_rect(rect, label, style::TEXT, FONT_SCALE * TYPE_LABEL);
    clicked
}

const COLOR_REC: Color = Color::rgb(0.92, 0.22, 0.28);

/// Recording status pill (timer + frame count).
pub fn recording_pill(ui: &mut GuiContext<'_>, rect: Rect, seconds: f32, frames: usize) {
    ui.panel_rounded(rect, style::SURFACE, style::RADIUS_PILL);
    ui.panel_rounded(
        Rect::from_pos_size(rect.min_x + 8.0, rect.min_y + (rect.height() - 8.0) * 0.5, 8.0, 8.0),
        COLOR_REC,
        4.0,
    );
    let m = seconds as u32 / 60;
    let s = seconds as u32 % 60;
    ui.label_at(
        rect.min_x + 22.0,
        rect.min_y + 6.0,
        &format!("REC {m:02}:{s:02}  {frames}f"),
        style::TEXT,
        FONT_SCALE * TYPE_LABEL,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_view_roundtrip() {
        let view = TimeView { t0: 0.0, t1: 2.0 };
        let lane = Rect::from_pos_size(100.0, 0.0, 200.0, 20.0);
        let x = view.time_to_x(1.0, lane);
        let t = view.x_to_time(x, lane);
        assert!((t - 1.0).abs() < 1e-4);
    }
}
