// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::imgui::*;
use crate::imutil;
use crate::model;
use memol::*;
use std::*;

pub struct Sequencer {
    pub scroll_ratio: f32,
    dragged: bool,
    time_scale: f32,
    line_width: f32,
    color_line_dark: u32,
    color_line_light: u32,
    color_note_bg_normal: u32,
    color_note_bg_active: u32,
    color_note_fg_normal: u32,
    color_note_fg_active: u32,
    color_hovered_vert: u32,
    color_hovered_horz: u32,
}

impl Sequencer {
    pub fn new() -> Self {
        Sequencer {
            scroll_ratio: 0.0,
            dragged: false,
            time_scale: 24.0,
            line_width: 0.25,
            color_line_dark: imutil::pack_color(ImVec4::new(1.0, 1.0, 1.0, 8.0) / 12.0),
            color_line_light: imutil::pack_color(ImVec4::new(1.0, 1.0, 1.0, 4.0) / 12.0),
            color_note_bg_normal: imutil::pack_color(ImVec4::new(1.0, 1.0, 1.0, 6.0) / 12.0),
            color_note_bg_active: imutil::pack_color(ImVec4::new(12.0, 6.0, 1.0, 6.0) / 12.0),
            color_note_fg_normal: imutil::pack_color(ImVec4::new(1.0, 1.0, 1.0, 12.0) / 12.0),
            color_note_fg_active: imutil::pack_color(ImVec4::new(12.0, 6.0, 1.0, 12.0) / 12.0),
            color_hovered_vert: imutil::pack_color(ImVec4::new(12.0, 6.0, 1.0, 12.0) / 12.0),
            color_hovered_horz: imutil::pack_color(ImVec4::new(12.0, 6.0, 1.0, 6.0) / 12.0),
        }
    }

    pub unsafe fn draw(&mut self, model: &mut model::Model, size: ImVec2) -> bool {
        let mut changed = false;
        let (playing, loc) = model.player.status();
        let time_len = model.assembly.len.to_float() as f32;
        let time_cur = (loc * model.tempo) as f32;

        let content_h = size.y - get_style().ScrollbarSize;
        let unit = content_h / 128.0;
        let content_w = unit * self.time_scale * (time_len + 1.0);

        SetNextWindowContentSize(&ImVec2::new(content_w, content_h));
        BeginChild(
            c_str!("sequencer"),
            &size,
            false,
            ImGuiWindowFlags_AlwaysHorizontalScrollbar as i32,
        );
        // scroll.
        if IsMouseDragging(1, -1.0) {
            let a = 15.0 * get_io().DeltaTime;
            SetScrollX(GetScrollX() + a * GetMouseDragDelta(1, -1.0).x);
        } else if model.follow && playing {
            let next = (time_cur + 0.5) * self.time_scale * unit - (1.0 / 6.0) * size.x;
            let a = f32::exp(-2.0 * get_io().DeltaTime);
            SetScrollX(a * GetScrollX() + (1.0 - a) * next);
        }

        let mut ctx = imutil::DrawContext::new(unit, ImVec2::new(0.5 * unit * self.time_scale, 0.0));

        // seek or copy.
        if IsWindowHovered(0) && IsMouseReleased(1) && !self.dragged {
            if model.copying_notes.is_empty() {
                let x = ctx.to_local(GetMousePos()).x / self.time_scale;
                let x = f32::min(f32::max(x, 0.0), time_len);
                model.player.seek(x as f64 / model.tempo);
                changed = true;
            } else {
                model.copy_notes_to_clipboard();
                model.note_off_all();
            }
        }

        // render.
        self.draw_indicator(&mut ctx, model, time_len);
        self.draw_background(&mut ctx, time_len);
        for (n, ch) in model.assembly.channels.iter() {
            if !model.channel_subs[*n] || *n == model.channel_main {
                continue;
            }
            self.draw_notes(
                &mut ctx,
                model,
                &ch.score,
                time_cur,
                self.color_note_bg_normal,
                self.color_note_bg_active,
            );
        }
        for (n, ch) in model.assembly.channels.iter() {
            if *n != model.channel_main {
                continue;
            }
            self.draw_notes(
                &mut ctx,
                model,
                &ch.score,
                time_cur,
                self.color_note_fg_normal,
                self.color_note_fg_active,
            );
        }
        self.draw_time_bar(&mut ctx, time_cur);

        self.dragged = IsMouseDragging(1, -1.0);
        self.scroll_ratio = if GetScrollMaxX() > 0.0 {
            GetScrollX() / GetScrollMaxX()
        } else {
            0.5
        };
        EndChild();

        changed
    }

    unsafe fn draw_indicator(&mut self, ctx: &mut imutil::DrawContext, model: &mut model::Model, time_len: f32) {
        for y in 0..128 {
            let v0 = ImVec2::new(self.time_scale * 0.0, y as f32);
            let v1 = ImVec2::new(self.time_scale * time_len, y as f32 + 1.0);
            ctx.add_dummy(v0, v1);
            if IsItemHovered(0) && IsMouseClicked(0, false) {
                model.copying_notes.push(y);
                model.note_on(y as u8);
            }
            if IsItemHovered(0) || model.on_notes[y as usize] {
                ctx.add_rect_filled(v0, v1, self.color_hovered_horz, 0.0, !0);
            }
        }
        if model.copying_notes.len() > 0 {
            BeginTooltip();
            imutil::show_text(&model.note_symbols(&model.copying_notes));
            EndTooltip();
        }
    }

    fn draw_background(&mut self, ctx: &mut imutil::DrawContext, time_len: f32) {
        // vertical lines.
        for i in 0..time_len.floor() as i32 + 1 {
            #[rustfmt::skip]
            let ys = [
                (43 - 24, 57 - 24),
                (43     , 57     ),
                (64     , 77     ),
                (64 + 24, 77 + 24),
            ];
            for &(y0, y1) in ys.iter() {
                let v0 = ImVec2::new(self.time_scale * i as f32, y0 as f32 + 0.5);
                let v1 = ImVec2::new(self.time_scale * i as f32, y1 as f32 + 0.5);
                ctx.add_line(v0, v1, self.color_line_dark, self.line_width);
            }
        }

        // bold horizontal lines.
        #[rustfmt::skip]
        let ys = [
            43 - 24, 47 - 24, 50 - 24, 53 - 24, 57 - 24,
            43,      47,      50,      53,      57,
            64,      67,      71,      74,      77,
            64 + 24, 67 + 24, 71 + 24, 74 + 24, 77 + 24,
        ];
        for &y in ys.iter() {
            let v0 = ImVec2::new(self.time_scale * 0.0 - 0.5 * self.line_width, y as f32 + 0.5);
            let v1 = ImVec2::new(self.time_scale * time_len + 0.5 * self.line_width, y as f32 + 0.5);
            ctx.add_line(v0, v1, self.color_line_dark, self.line_width);
        }

        // thin horizontal lines.
        #[rustfmt::skip]
        let ys = [
            36,      40,
            60,
            81,      84,
            81 + 24, 84 + 24,
        ];
        for &y in ys.iter() {
            let v0 = ImVec2::new(self.time_scale * 0.0 - 0.5 * self.line_width, y as f32 + 0.5);
            let v1 = ImVec2::new(self.time_scale * time_len + 0.5 * self.line_width, y as f32 + 0.5);
            ctx.add_line(v0, v1, self.color_line_light, self.line_width);
        }
    }

    unsafe fn draw_notes(
        &self,
        ctx: &mut imutil::DrawContext,
        model: &model::Model,
        ir: &generator::ScoreIr,
        time_cur: f32,
        color_0: u32,
        color_1: u32,
    ) {
        for note in ir.iter() {
            let nnum = match note.nnum {
                Some(v) => v,
                None => continue,
            };

            let t0 = note.t0.to_float() as f32;
            let t1 = note.t1.to_float() as f32;
            let x0 = ImVec2::new(self.time_scale * t0, nnum as f32 + 0.0);
            let x1 = ImVec2::new(self.time_scale * t1, nnum as f32 + 1.0);
            let color = if t0 <= time_cur && time_cur <= t1 {
                color_1
            } else {
                color_0
            };
            ctx.add_rect_filled(x0, x1, color, 0.5, !0);

            ctx.add_dummy(x0, x1);
            if IsItemHovered(0) {
                BeginTooltip();
                imutil::show_text(&format!("     note = {}", model.note_symbol(nnum)));
                imutil::show_text(&format!(
                    "gate time = {} + {}/{}",
                    misc::idiv(note.t0.y, note.t0.x),
                    misc::imod(note.t0.y, note.t0.x),
                    note.t0.x,
                ));
                let dt = note.t1 - note.t0;
                imutil::show_text(&format!(" duration = {}/{}", dt.y, dt.x));
                EndTooltip();
            }
        }
    }

    unsafe fn draw_time_bar(&self, ctx: &mut imutil::DrawContext, time_cur: f32) {
        let v0 = ImVec2::new(self.time_scale * time_cur, 0.0);
        let v1 = ImVec2::new(self.time_scale * time_cur, 128.0);
        ctx.add_line(v0, v1, self.color_hovered_vert, self.line_width);
    }
}
