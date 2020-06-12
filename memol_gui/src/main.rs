// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
#![windows_subsystem = "windows"]
#[macro_use]
mod imutil;
mod compile_thread;
mod imgui;
mod main_widget;
mod model;
mod renderer;
mod sequencer_widget;
mod window;
use gumdrop::Options;
use memol::*;
use memol_cli::player::PlayerExt;
use memol_cli::{player, player_jack, player_net};
use std::*;

const JACK_FRAME_WAIT: i32 = 12;

#[derive(gumdrop::Options)]
struct ArgOptions {
    #[options(free)]
    file: Option<path::PathBuf>,
    #[options(help = "Set a background image.", meta = "FILE")]
    wallpaper: Option<path::PathBuf>,
    #[options(help = "Use JACK.")]
    jack: bool,
    #[options(help = "Use plugins.")]
    plugin: bool,
    #[options(help = "Accept remote connections.")]
    any: bool,
    #[options(help = "Connect to specified ports.", meta = "PORT")]
    connect: Vec<String>,
}

#[derive(Debug)]
enum UiMessage {
    Data(path::PathBuf, String, Assembly, Vec<midi::Event>),
    Text(String),
    Midi(Vec<midi::Event>),
}

fn init_imgui(scale: f32) -> main_widget::Fonts {
    let io = imgui::get_io();
    imutil::set_theme(
        imgui::ImVec4::new(1.0, 1.0, 1.0, 12.0) / 12.0,
        imgui::ImVec4::new(1.0, 1.0, 1.0, 1.0),
        imgui::ImVec4::new(1.0, 1.0, 1.0, 24.0) / 24.0,
    );
    unsafe {
        imgui::get_style().FramePadding = imgui::ImVec2::new(4.0, 4.0);
        imgui::get_style().ItemSpacing = imgui::ImVec2::new(4.0, 4.0);
        imgui::get_style().ScaleAllSizes(scale);

        let mut cfg = imgui::ImFontConfig::new();
        cfg.FontDataOwnedByAtlas = false;
        cfg.OversampleH = 4;
        cfg.GlyphOffset.y = 0.0;
        let font = include_bytes!("../fonts/SourceSansPro-Regular.ttf");
        let sans = (*io.Fonts).AddFontFromMemoryTTF(
            font.as_ptr() as *mut os::raw::c_void,
            font.len() as i32,
            (18.0 * scale).round(),
            &cfg,
            [0x20, 0xff, 0x2026, 0x2027, 0].as_ptr(),
        );
        let font = include_bytes!("../fonts/inconsolata_regular.ttf");
        let mono = (*io.Fonts).AddFontFromMemoryTTF(
            font.as_ptr() as *mut os::raw::c_void,
            font.len() as i32,
            (18.0 * scale).round(),
            &cfg,
            [0x20, 0xff, 0x2026, 0x2027, 0].as_ptr(),
        );
        let font = include_bytes!("../fonts/awesome_solid.ttf");
        let icon = (*io.Fonts).AddFontFromMemoryTTF(
            font.as_ptr() as *mut os::raw::c_void,
            font.len() as i32,
            (18.0 * scale).round(),
            &cfg,
            [0xf000, 0xf7ff, 0].as_ptr(),
        );
        main_widget::Fonts {
            sans: sans,
            mono: mono,
            icon: icon,
        }
    }
}

fn lighten_image(img: &mut image::RgbaImage, ratio: f32) {
    for px in img.pixels_mut() {
        let rgb = imgui::ImVec4::new(px[0] as f32, px[1] as f32, px[2] as f32, 0.0);
        let rgb = imutil::srgb_gamma_to_linear((1.0 / 255.0) * rgb);
        let rgb = imgui::ImVec4::constant(1.0 - ratio) + ratio * rgb;
        let rgb = (255.0 * imutil::srgb_linear_to_gamma(rgb)).round();
        px[0] = rgb.x as u8;
        px[1] = rgb.y as u8;
        px[2] = rgb.z as u8;
    }
}

fn main() {
    || -> Result<(), Box<dyn error::Error>> {
        // parse the command line.
        let args: Vec<_> = env::args().collect();
        let opts = match ArgOptions::parse_args_default(&args[1..]) {
            Ok(e) => e,
            Err(_) => return Err(ArgOptions::usage().into()),
        };

        // create instances.
        let mut compiler = compile_thread::CompileThread::new();
        let model = rc::Rc::new(cell::RefCell::new(model::Model::new(compiler.create_sender())));
        let mut widget = main_widget::MainWidget::new();
        let mut window = window::Window::new("memol")?;

        // initialize a window.
        let fonts = init_imgui(window.hidpi_factor() as f32);
        window.update_font();
        if let Some(Ok(img)) = opts.wallpaper.map(image::open) {
            let mut img = img.to_rgba();
            lighten_image(&mut img, 0.5);
            let mut wallpaper = renderer::Texture::new();
            unsafe { wallpaper.upload_u32(img.as_ptr(), img.width() as i32, img.height() as i32) };
            widget.wallpaper = Some(wallpaper);
        } else {
            window.set_background(imgui::ImVec4::constant(1.0));
        }

        window.on_draw({
            let model = model.clone();
            move || {
                let changed = unsafe { widget.draw(&mut model.borrow_mut(), &fonts) };
                if changed {
                    JACK_FRAME_WAIT
                } else {
                    0
                }
            }
        });
        window.on_message({
            let model = model.clone();
            move |msg| {
                match msg {
                    UiMessage::Data(path, code, asm, evs) => {
                        model.borrow_mut().set_data(path, code, asm, evs);
                    }
                    UiMessage::Text(text) => {
                        model.borrow_mut().text = Some(text);
                    }
                    UiMessage::Midi(evs) => {
                        model.borrow_mut().handle_midi_inputs(&evs);
                    }
                }
                JACK_FRAME_WAIT
            }
        });
        window.on_file_dropped({
            let model = model.clone();
            move |path| {
                model
                    .borrow_mut()
                    .compile_tx
                    .send(compile_thread::Message::File(path.clone()))
                    .unwrap();
                0
            }
        });

        // initialize a player.
        let addr = (
            if opts.any {
                net::Ipv6Addr::UNSPECIFIED
            } else {
                net::Ipv6Addr::LOCALHOST
            },
            27182,
        );
        let mut player: Box<dyn player::Player> = match (opts.jack, opts.plugin) {
            (true, false) => Box::new(player_jack::Player::new("memol")?),
            (false, true) => Box::new(player_net::Player::new(addr)?),
            _ => {
                #[cfg(all(target_family = "unix", not(target_os = "macos")))]
                let player = player_jack::Player::new("memol");
                #[cfg(not(all(target_family = "unix", not(target_os = "macos"))))]
                let player = player_net::Player::new(addr);
                Box::new(player?)
            }
        };
        player.on_received({
            let window_tx = window.create_proxy();
            move |evs| window_tx.send_event(UiMessage::Midi(evs.to_vec())).unwrap()
        });
        for port in opts.connect {
            player.connect_to(&port)?;
        }
        model.borrow_mut().player = player;

        // initialize a compiler.
        compiler.on_success({
            let window_tx = window.create_proxy();
            move |path, asm, evs| {
                let code = fs::read_to_string(&path).unwrap_or_else(|_| String::new());
                window_tx.send_event(UiMessage::Data(path, code, asm, evs)).unwrap();
            }
        });
        compiler.on_failure({
            let window_tx = window.create_proxy();
            move |text| {
                window_tx.send_event(UiMessage::Text(text)).unwrap();
            }
        });
        if let Some(path) = opts.file {
            compiler
                .create_sender()
                .send(compile_thread::Message::File(path.into()))
                .unwrap();
        } else {
            window
                .create_proxy()
                .send_event(UiMessage::Text("Drag and drop to open a file.".into()))
                .unwrap();
        }
        compiler.spawn();

        window.run();
        Ok(())
    }()
    .unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        process::exit(-1);
    });
}
