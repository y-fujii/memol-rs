// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use crate::imgui::*;
use crate::imutil;
use crate::renderer;
use crate::compile_thread;
use crate::model;
use crate::piano_roll;


pub struct MainWidget {
	pub wallpaper: Option<renderer::Texture>,
	ports_from: Vec<(String, bool)>,
	ports_to: Vec<(String, bool)>,
	piano_roll: piano_roll::PianoRoll,
}

impl MainWidget {
	pub fn new() -> Self {
		MainWidget{
			wallpaper: None,
			ports_from: Vec::new(),
			ports_to: Vec::new(),
			piano_roll: piano_roll::PianoRoll::new(),
		}
	}

	pub unsafe fn draw( &mut self, model: &mut model::Model ) -> bool {
		if let Some( ref text ) = model.text {
			if imutil::message_dialog( "Message", text ) {
				model.text = None;
			}
		}

		let changed = self.draw_transport( model );

		imutil::root_begin( 0 );
			let size = GetWindowSize();
			match model.mode {
				model::DisplayMode::PianoRoll => self.piano_roll.draw( model, size ),
				model::DisplayMode::Code      => self.draw_code( model, size ),
			}

			if let Some( ref wallpaper ) = self.wallpaper {
				let scale = f32::max( size.x / wallpaper.size.0 as f32, size.y / wallpaper.size.1 as f32 );
				let wsize = scale * ImVec2::new( wallpaper.size.0 as f32, wallpaper.size.1 as f32 );
				let v0 = GetWindowPos() + self.piano_roll.scroll_ratio * (size - wsize);
				(*GetWindowDrawList()).AddImage(
					wallpaper.id as _, &v0, &(v0 + wsize), &ImVec2::zero(), &ImVec2::new( 1.0, 1.0 ), 0xffff_ffff,
				);
			}
		imutil::root_end();

		changed || model.player.is_playing()
	}

	unsafe fn draw_code( &mut self, model: &mut model::Model, size: ImVec2 ) {
		BeginChild(
			c_str!( "code" ), &size, false,
			(ImGuiWindowFlags_AlwaysUseWindowPadding | ImGuiWindowFlags_HorizontalScrollbar) as i32,
		);
			PushStyleColor( ImGuiCol_Text as i32, 0xff00_0000 );
				imutil::show_text( &model.code );
			PopStyleColor( 1 );
		EndChild();
	}

	unsafe fn draw_transport( &mut self, model: &mut model::Model ) -> bool {
		let mut changed = false;

		let padding = get_style().WindowPadding;
		PushStyleVar1( ImGuiStyleVar_WindowPadding as i32, &(0.5 * padding).round() );
		SetNextWindowPos( &ImVec2::zero(), ImGuiCond_Always as i32, &ImVec2::zero() );
		Begin(
			c_str!( "Transport" ), ptr::null_mut(),
			(ImGuiWindowFlags_AlwaysAutoResize | ImGuiWindowFlags_NoMove |
			 ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoTitleBar) as i32
		);
			let size = ImVec2::new( GetFontSize() * 2.0, 0.0 );
			if Button( c_str!( "\u{f048}" ), &size ) {
				model.player.seek( 0.0 ).ok();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04b}" ), &size ) {
				model.player.play().ok();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04d}" ), &size ) {
				model.player.stop().ok();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f051}" ), &size ) {
				model.player.seek( model.assembly.len.to_float() / model.tempo ).ok();
				changed = true;
			}

			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Follow" ), &mut model.follow );
			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Autoplay" ), &mut model.autoplay );

			let mode_str = |mode| match mode {
				model::DisplayMode::PianoRoll => "Piano roll",
				model::DisplayMode::Code      => "Code",
			};
			SameLine( 0.0, -1.0 );
			PushItemWidth( imutil::text_size( "_Piano roll____" ).x );
			if BeginCombo( c_str!( "##mode" ), c_str!( "{}", mode_str( model.mode ) ), 0 ) {
				for &mode in [ model::DisplayMode::PianoRoll, model::DisplayMode::Code ].iter() {
					if Selectable( c_str!( "{}", mode_str( mode ) ), model.mode == mode, 0, &ImVec2::zero() ) {
						model.mode = mode;
					}
				}
				EndCombo();
			}
			PopItemWidth();

			SameLine( 0.0, -1.0 );
			PushItemWidth( imutil::text_size( "_Channel 00____" ).x );
			if BeginCombo( c_str!( "##channel" ), c_str!( "Channel {:2}", model.channel ), 0 ) {
				for &(i, _) in model.assembly.channels.iter() {
					if Selectable( c_str!( "Channel {:2}", i ), i == model.channel, 0, &ImVec2::zero() ) {
						model.channel = i;
						changed = true;
					}
				}
				EndCombo();
			}
			PopItemWidth();

			// ports from.
			SameLine( 0.0, -1.0 );
			if Button( c_str!( "Input from..." ), &ImVec2::zero() ) {
				OpenPopup( c_str!( "ports from" ) );
				self.ports_from = model.player.ports_from().unwrap_or_default();
			}
			if BeginPopup( c_str!( "ports from" ), 0 ) {
				for &mut (ref port, ref mut is_conn) in self.ports_from.iter_mut() {
					if Checkbox( c_str!( "{}", port ), is_conn ) {
						*is_conn = if *is_conn {
							model.player.connect_from( port ).is_ok()
						}
						else {
							model.player.disconnect_from( port ).is_err()
						}
					}
				}
				EndPopup();
			}

			// ports to.
			SameLine( 0.0, -1.0 );
			if Button( c_str!( "Output to..." ), &ImVec2::zero() ) {
				OpenPopup( c_str!( "ports to" ) );
				self.ports_to = model.player.ports_to().unwrap_or_default();
			}
			if BeginPopup( c_str!( "ports to" ), 0 ) {
				for &mut (ref port, ref mut is_conn) in self.ports_to.iter_mut() {
					if Checkbox( c_str!( "{}", port ), is_conn ) {
						*is_conn = if *is_conn {
							model.player.connect_to( port ).is_ok()
						}
						else {
							model.player.disconnect_to( port ).is_err()
						}
					}
				}
				EndPopup();
			}

			SameLine( 0.0, -1.0 );
			if Button( c_str!( "Generate SMF" ), &ImVec2::zero() ) {
				if let Err( e ) = model.generate_smf() {
					model.text = Some( format!( "{}", e ) );
				}
			}
			SameLine( 0.0, -1.0 );
			if Button( c_str!( "\u{f021}" ), &ImVec2::zero() ) {
				model.compile_tx.send( compile_thread::Message::Refresh ).unwrap();
			}
		End();
		PopStyleVar( 1 );

		changed
	}
}
