// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::imgui::*;
use crate::imutil;
use crate::model;
use crate::renderer;
use crate::sequencer_widget;
use std::*;

pub struct Fonts {
    pub sans: *mut ImFont,
    pub mono: *mut ImFont,
    pub icon: *mut ImFont,
}

pub struct MainWidget {
    pub wallpaper: Option<renderer::Texture>,
    ports_from: Option<Vec<(String, bool)>>,
    ports_to: Option<Vec<(String, bool)>>,
    sequencer: sequencer_widget::Sequencer,
}

impl MainWidget {
    pub fn new() -> Self {
        MainWidget {
            wallpaper: None,
            ports_from: None,
            ports_to: None,
            sequencer: sequencer_widget::Sequencer::new(),
        }
    }

    pub unsafe fn draw(&mut self, model: &mut model::Model, fonts: &Fonts) -> bool {
        if let Some(ref text) = model.text {
            if imutil::message_dialog("Message", text) {
                model.text = None;
            }
        }

        let size = get_io().DisplaySize;
        let size_l = ImVec2::new(f32::round(8.0 * GetFontSize()), size.y);
        let size_r = ImVec2::new(size.x - size_l.x, size.y);

        let mut changed = self.draw_transport(model, ImVec2::zero(), size_l, fonts);

        imutil::root_begin(
            "sequencer\0",
            ImVec2::new(size_l.x, 0.0),
            size_r,
            false,
            ImGuiWindowFlags_NoBackground,
        );

        PushFont(fonts.mono);
        changed |= match model.mode {
            model::DisplayMode::Sequencer => self.sequencer.draw(model, size_r),
            model::DisplayMode::Code => self.draw_code(model, size_r),
        };
        PopFont();

        if let Some(ref wallpaper) = self.wallpaper {
            let scale = f32::max(size_r.x / wallpaper.size.0 as f32, size_r.y / wallpaper.size.1 as f32);
            let wsize = scale * ImVec2::new(wallpaper.size.0 as f32, wallpaper.size.1 as f32);
            let v0 = GetWindowPos() + self.sequencer.scroll_ratio * (size_r - wsize);
            (*GetWindowDrawList()).AddImage(
                wallpaper.id as _,
                &v0,
                &(v0 + wsize),
                &ImVec2::zero(),
                &ImVec2::new(1.0, 1.0),
                0xffff_ffff,
            );
        }

        End();

        changed || model.player.status().0
    }

    unsafe fn draw_code(&mut self, model: &mut model::Model, size: ImVec2) -> bool {
        BeginChild(
            c_str!("code"),
            &size,
            false,
            (ImGuiWindowFlags_AlwaysUseWindowPadding | ImGuiWindowFlags_HorizontalScrollbar) as i32,
        );
        PushStyleColor(ImGuiCol_Text as i32, 0xff00_0000);
        imutil::show_text(&model.code);
        PopStyleColor(1);
        EndChild();

        false
    }

    unsafe fn draw_transport(&mut self, model: &mut model::Model, pos: ImVec2, size: ImVec2, fonts: &Fonts) -> bool {
        let mut changed = false;

        imutil::root_begin("transport\0", pos, size, true, 0);
        PushItemWidth(-1.0);

        let (_, time) = model.player.status();
        Text(c_str!("Time: {:.02} sec.", time));

        PushFont(fonts.icon);
        let size = ImVec2::new(GetWindowContentRegionWidth() / 2.0 - 1.0, 0.0);
        if Button(c_str!("\u{f04b}"), &size) {
            model.player.play();
            changed = true;
        }
        SameLine(0.0, 1.0);
        if Button(c_str!("\u{f04d}"), &size) {
            model.player.stop();
            changed = true;
        }
        if Button(c_str!("\u{f048}"), &size) {
            model.player.seek(0.0);
            changed = true;
        }
        SameLine(0.0, 1.0);
        if Button(c_str!("\u{f051}"), &size) {
            model.player.seek(model.assembly.len.to_float() / model.tempo);
            changed = true;
        }
        PopFont();

        if Button(c_str!("Generate SMF"), &ImVec2::new(-1.0, 0.0)) {
            if let Err(e) = model.generate_smf() {
                model.text = Some(format!("{}", e));
            }
        }

        if Button(c_str!("I/O ports\u{2026}"), &ImVec2::new(-1.0, 0.0)) {
            OpenPopup(c_str!("ports"), 0);
            self.ports_from = model.player.ports_from().ok();
            self.ports_to = model.player.ports_to().ok();
        }
        if BeginPopup(c_str!("ports"), 0) {
            PushFont(fonts.icon);
            Text(c_str!("\u{f7c0}"));
            PopFont();
            SameLine(0.0, -1.0);
            Text(c_str!("{}", model.player.info()));

            if let Some(ports) = &mut self.ports_from {
                Spacing();
                Separator();
                Text(c_str!("Input from\u{2026}"));
                for &mut (ref port, ref mut is_conn) in ports.iter_mut() {
                    if Checkbox(c_str!("{}", port), is_conn) {
                        *is_conn = if *is_conn {
                            model.player.connect_from(port).is_ok()
                        } else {
                            model.player.disconnect_from(port).is_err()
                        }
                    }
                }
            }

            if let Some(ports) = &mut self.ports_to {
                Spacing();
                Separator();
                Text(c_str!("Output to\u{2026}"));
                for &mut (ref port, ref mut is_conn) in ports.iter_mut() {
                    if Checkbox(c_str!("{}", port), is_conn) {
                        *is_conn = if *is_conn {
                            model.player.connect_to(port).is_ok()
                        } else {
                            model.player.disconnect_to(port).is_err()
                        }
                    }
                }
            }

            EndPopup();
        }

        Checkbox(c_str!("Follow"), &mut model.follow);
        Checkbox(c_str!("Autoplay"), &mut model.autoplay);

        Spacing();
        Separator();
        Text(c_str!("Display mode"));

        if RadioButton(c_str!("Score"), model.mode == model::DisplayMode::Sequencer) {
            model.mode = model::DisplayMode::Sequencer;
        }
        if RadioButton(c_str!("Score + Plot"), false) {}
        if RadioButton(c_str!("Code"), model.mode == model::DisplayMode::Code) {
            model.mode = model::DisplayMode::Code;
        }

        Spacing();
        Separator();
        Text(c_str!("Channels"));

        for &(i, _) in model.assembly.channels.iter() {
            if RadioButton(c_str!("##radio_{:02}", i), model.channel_main == i) {
                model.channel_main = i;
            }
            SameLine(0.0, -1.0);
            Checkbox(c_str!("#{:02}", i), &mut model.channel_subs[i]);
        }

        PopItemWidth();
        End();

        changed
    }
}
