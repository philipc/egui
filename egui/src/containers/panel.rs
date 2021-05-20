//! Panels are fixed [`Ui`] regions.
//!
//! Together with [`Window`] and [`Area`]:s they are
//! the only places where you can put you widgets.
//!
//! The order in which you add panels matter!
//!
//! Add [`CentralPanel`] and [`Window`]:s last.

use std::ops::RangeInclusive;

use crate::*;

#[derive(Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
struct PanelState {
    rect: Rect,
}

// ----------------------------------------------------------------------------

/// A panel that covers the entire left side of the screen.
///
/// `SidePanel`s must be added before adding any [`CentralPanel`] or [`Window`]s.
///
/// ```
/// # let mut ctx = egui::CtxRef::default();
/// # ctx.begin_frame(Default::default());
/// # let ctx = &ctx;
/// egui::SidePanel::left("my_side_panel", 0.0).show(ctx, |ui| {
///    ui.label("Hello World!");
/// });
/// ```
#[must_use = "You should call .show()"]
pub struct SidePanel {
    id: Id,
    frame: Option<Frame>,
    resizable: bool,
    default_width: f32,
    width_range: RangeInclusive<f32>,
}

impl SidePanel {
    /// `id_source`: Something unique, e.g. `"my_side_panel"`.
    pub fn left(id_source: impl std::hash::Hash) -> Self {
        Self {
            id: Id::new(id_source),
            frame: None,
            resizable: true,
            default_width: 200.0,
            width_range: 96.0..=f32::INFINITY,
        }
    }

    /// Switch resizable on/off.
    /// Default is `true`.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// The initial wrapping width of the `SidePanel`.
    pub fn default_width(mut self, default_width: f32) -> Self {
        self.default_width = default_width;
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.width_range = min_width..=(*self.width_range.end());
        self
    }

    pub fn max_width(mut self, max_width: f32) -> Self {
        self.width_range = (*self.width_range.start())..=max_width;
        self
    }

    /// The allowable width range for resizable panels.
    pub fn width_range(mut self, width_range: RangeInclusive<f32>) -> Self {
        self.width_range = width_range;
        self
    }

    /// Change the background color, margins, etc.
    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }
}

impl SidePanel {
    pub fn show<R>(
        self,
        ctx: &CtxRef,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let Self {
            id,
            frame,
            resizable,
            default_width,
            width_range,
        } = self;

        let mut panel_rect = ctx.available_rect();
        {
            let mut width = default_width;
            if let Some(state) = ctx.memory().id_data.get::<PanelState>(&id) {
                width = state.rect.width();
            }
            width = clamp_to_range(width, width_range.clone());
            panel_rect.max.x = panel_rect.min.x + width;
        }

        let mut resize_hover = false;
        let mut is_resizing = false;
        if resizable {
            let resize_id = id.with("__resize");
            if let Some(pointer) = ctx.input().pointer.latest_pos() {
                resize_hover = panel_rect.y_range().contains(&pointer.y)
                    && (panel_rect.right() - pointer.x).abs()
                        <= ctx.style().interaction.resize_grab_radius_side;

                if ctx.input().pointer.any_pressed()
                    && ctx.input().pointer.any_down()
                    && resize_hover
                {
                    ctx.memory().interaction.drag_id = Some(resize_id);
                }
                is_resizing = ctx.memory().interaction.drag_id == Some(resize_id);
                if is_resizing {
                    let width = pointer.x - panel_rect.left();
                    let width = clamp_to_range(width, width_range);
                    panel_rect.max.x = panel_rect.min.x + width;
                }

                if resize_hover || is_resizing {
                    ctx.output().cursor_icon = CursorIcon::ResizeHorizontal;
                }
            }
        }

        let layer_id = LayerId::background();

        let clip_rect = ctx.input().screen_rect();
        let mut panel_ui = Ui::new(ctx.clone(), layer_id, id, panel_rect, clip_rect);

        let frame = frame.unwrap_or_else(|| Frame::side_top_panel(&ctx.style()));
        let inner_response = frame.show(&mut panel_ui, |ui| {
            ui.set_min_height(ui.max_rect_finite().height()); // Make sure the frame fills the full height
            add_contents(ui)
        });

        let rect = inner_response.response.rect;

        if resize_hover || is_resizing {
            let stroke = if is_resizing {
                ctx.style().visuals.widgets.active.bg_stroke
            } else {
                ctx.style().visuals.widgets.hovered.bg_stroke
            };
            // use foreground_painter so the resize line won't be covered by subsequent panels
            ctx.foreground_painter()
                .line_segment([rect.right_top(), rect.right_bottom()], stroke);
        }

        // Only inform ctx about what we actually used, so we can shrink the native window to fit.
        ctx.frame_state().allocate_left_panel(rect);

        ctx.memory().id_data.insert(id, PanelState { rect });

        inner_response
    }
}

// ----------------------------------------------------------------------------

/// A panel that covers the entire top side of the screen.
///
/// `TopPanel`s must be added before adding any [`CentralPanel`] or [`Window`]s.
///
/// ```
/// # let mut ctx = egui::CtxRef::default();
/// # ctx.begin_frame(Default::default());
/// # let ctx = &ctx;
/// egui::TopPanel::top("my_top_panel").show(ctx, |ui| {
///    ui.label("Hello World!");
/// });
/// ```
#[must_use = "You should call .show()"]
pub struct TopPanel {
    id: Id,
    max_height: Option<f32>,
    frame: Option<Frame>,
}

impl TopPanel {
    /// `id_source`: Something unique, e.g. `"my_top_panel"`.
    /// Default height is that of `interact_size.y` (i.e. a button),
    /// but the panel will expand as needed.
    pub fn top(id_source: impl std::hash::Hash) -> Self {
        Self {
            id: Id::new(id_source),
            max_height: None,
            frame: None,
        }
    }

    /// Change the background color, margins, etc.
    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }
}

impl TopPanel {
    pub fn show<R>(
        self,
        ctx: &CtxRef,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let Self {
            id,
            max_height,
            frame,
        } = self;
        let max_height = max_height.unwrap_or_else(|| ctx.style().spacing.interact_size.y);

        let mut panel_rect = ctx.available_rect();
        panel_rect.max.y = panel_rect.max.y.at_most(panel_rect.min.y + max_height);

        let layer_id = LayerId::background();

        let clip_rect = ctx.input().screen_rect();
        let mut panel_ui = Ui::new(ctx.clone(), layer_id, id, panel_rect, clip_rect);

        let frame = frame.unwrap_or_else(|| Frame::side_top_panel(&ctx.style()));
        let inner_response = frame.show(&mut panel_ui, |ui| {
            ui.set_min_width(ui.max_rect_finite().width()); // Make the frame fill full width
            add_contents(ui)
        });

        // Only inform ctx about what we actually used, so we can shrink the native window to fit.
        ctx.frame_state()
            .allocate_top_panel(inner_response.response.rect);

        inner_response
    }
}

// ----------------------------------------------------------------------------

/// A panel that covers the remainder of the screen,
/// i.e. whatever area is left after adding other panels.
///
/// `CentralPanel` must be added after all other panels.
/// Any [`Window`]s and [`Area`]s will cover the `CentralPanel`.
///
/// ```
/// # let mut ctx = egui::CtxRef::default();
/// # ctx.begin_frame(Default::default());
/// # let ctx = &ctx;
/// egui::CentralPanel::default().show(ctx, |ui| {
///    ui.label("Hello World!");
/// });
/// ```
#[must_use = "You should call .show()"]
#[derive(Default)]
pub struct CentralPanel {
    frame: Option<Frame>,
}

impl CentralPanel {
    /// Change the background color, margins, etc.
    pub fn frame(mut self, frame: Frame) -> Self {
        self.frame = Some(frame);
        self
    }
}

impl CentralPanel {
    pub fn show<R>(
        self,
        ctx: &CtxRef,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let Self { frame } = self;

        let panel_rect = ctx.available_rect();

        let layer_id = LayerId::background();
        let id = Id::new("central_panel");

        let clip_rect = ctx.input().screen_rect();
        let mut panel_ui = Ui::new(ctx.clone(), layer_id, id, panel_rect, clip_rect);

        let frame = frame.unwrap_or_else(|| Frame::central_panel(&ctx.style()));
        let inner_response = frame.show(&mut panel_ui, |ui| {
            ui.expand_to_include_rect(ui.max_rect()); // Expand frame to include it all
            add_contents(ui)
        });

        // Only inform ctx about what we actually used, so we can shrink the native window to fit.
        ctx.frame_state()
            .allocate_central_panel(inner_response.response.rect);

        inner_response
    }
}

fn clamp_to_range(x: f32, range: RangeInclusive<f32>) -> f32 {
    x.clamp(
        range.start().min(*range.end()),
        range.start().max(*range.end()),
    )
}
