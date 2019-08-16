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

        let mut changed = self.draw_transport(model, fonts);

        imutil::root_begin(0);

        PushFont(fonts.mono);
        let size = GetWindowSize();
        changed |= match model.mode {
            model::DisplayMode::Sequencer => self.sequencer.draw(model, size),
            model::DisplayMode::Code => self.draw_code(model, size),
        };
        PopFont();

        if let Some(ref wallpaper) = self.wallpaper {
            let scale = f32::max(size.x / wallpaper.size.0 as f32, size.y / wallpaper.size.1 as f32);
            let wsize = scale * ImVec2::new(wallpaper.size.0 as f32, wallpaper.size.1 as f32);
            let v0 = GetWindowPos() + self.sequencer.scroll_ratio * (size - wsize);
            (*GetWindowDrawList()).AddImage(
                wallpaper.id as _,
                &v0,
                &(v0 + wsize),
                &ImVec2::zero(),
                &ImVec2::new(1.0, 1.0),
                0xffff_ffff,
            );
        }
        imutil::root_end();

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

    unsafe fn draw_transport(&mut self, model: &mut model::Model, fonts: &Fonts) -> bool {
        let mut changed = false;

        let padding = get_style().WindowPadding;
        PushStyleVar1(ImGuiStyleVar_WindowPadding as i32, &(0.5 * padding).round());
        SetNextWindowPos(&ImVec2::zero(), ImGuiCond_Always as i32, &ImVec2::zero());
        Begin(
            c_str!("Transport"),
            ptr::null_mut(),
            (ImGuiWindowFlags_AlwaysAutoResize
                | ImGuiWindowFlags_NoMove
                | ImGuiWindowFlags_NoResize
                | ImGuiWindowFlags_NoTitleBar) as i32,
        );
        PushStyleVar1(ImGuiStyleVar_WindowPadding as i32, &padding);

        PushFont(fonts.icon);
        let size = ImVec2::new(GetFontSize() * 2.0, 0.0);
        if Button(c_str!("\u{f048}"), &size) {
            model.player.seek(0.0);
            changed = true;
        }
        SameLine(0.0, 1.0);
        if Button(c_str!("\u{f04b}"), &size) {
            model.player.play();
            changed = true;
        }
        SameLine(0.0, 1.0);
        if Button(c_str!("\u{f04d}"), &size) {
            model.player.stop();
            changed = true;
        }
        SameLine(0.0, 1.0);
        if Button(c_str!("\u{f051}"), &size) {
            model.player.seek(model.assembly.len.to_float() / model.tempo);
            changed = true;
        }
        PopFont();

        SameLine(0.0, -1.0);
        Checkbox(c_str!("Follow"), &mut model.follow);
        SameLine(0.0, -1.0);
        Checkbox(c_str!("Autoplay"), &mut model.autoplay);

        let mode_str = |mode| match mode {
            model::DisplayMode::Sequencer => "Sequencer",
            model::DisplayMode::Code => "Code",
        };
        SameLine(0.0, -1.0);
        SetNextItemWidth(imutil::text_size("_Sequencer____").x);
        if BeginCombo(c_str!("##mode"), c_str!("{}", mode_str(model.mode)), 0) {
            for &mode in [model::DisplayMode::Sequencer, model::DisplayMode::Code].iter() {
                if Selectable(c_str!("{}", mode_str(mode)), model.mode == mode, 0, &ImVec2::zero()) {
                    model.mode = mode;
                }
            }
            EndCombo();
        }

        SameLine(0.0, -1.0);
        SetNextItemWidth(imutil::text_size("_Channel 00____").x);
        if BeginCombo(c_str!("##channel"), c_str!("Channel {:2}", model.channel), 0) {
            for &(i, _) in model.assembly.channels.iter() {
                if Selectable(c_str!("Channel {:2}", i), i == model.channel, 0, &ImVec2::zero()) {
                    model.channel = i;
                    changed = true;
                }
            }
            EndCombo();
        }

        SameLine(0.0, -1.0);
        if Button(c_str!("I/O ports\u{2026}"), &ImVec2::zero()) {
            OpenPopup(c_str!("ports"));
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

        SameLine(0.0, -1.0);
        if Button(c_str!("Generate SMF"), &ImVec2::zero()) {
            if let Err(e) = model.generate_smf() {
                model.text = Some(format!("{}", e));
            }
        }

        PopStyleVar(1);
        End();
        PopStyleVar(1);

        changed
    }
}
