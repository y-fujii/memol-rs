// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use imutil;
use memol_cli::player;


pub enum Event {
	GenerateSmf,
	Refresh,
}

pub struct TransportWidget {
	pub events: Vec<Event>,
	pub channel: usize,
	pub follow: bool,
	pub autoplay: bool,
	ports_from: Vec<(String, bool)>,
	ports_to: Vec<(String, bool)>,
}

impl TransportWidget {
	pub fn new() -> Self {
		TransportWidget{
			events: Vec::new(),
			channel: 0,
			follow: true,
			autoplay: true,
			ports_from: Vec::new(),
			ports_to: Vec::new(),
		}
	}

	pub unsafe fn draw( &mut self, player: &player::Player, channels: &[usize] ) -> bool {
		use imgui::*;

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
				player.seek( 0.0 ).ok();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04b}" ), &size ) {
				player.play().ok();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04d}" ), &size ) {
				player.stop().ok();
				changed = true;
			}
			/*
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f051}" ), &size ) {
				player.seek_to_end();
				changed = true;
			}
			*/

			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Follow" ), &mut self.follow );
			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Autoplay" ), &mut self.autoplay );

			SameLine( 0.0, -1.0 );
			ImGui::PushItemWidth( imutil::text_size( "_Channel 00____" ).x );
			if BeginCombo( c_str!( "##channel" ), c_str!( "Channel {:2}", self.channel ), 0 ) {
				for &i in channels.iter() {
					if Selectable( c_str!( "Channel {:2}", i ), i == self.channel, 0, &ImVec2::zero() ) {
						self.channel = i;
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
				self.ports_from = player.ports_from().unwrap_or_default();
			}
			if BeginPopup( c_str!( "ports from" ), 0 ) {
				for &mut (ref port, ref mut is_conn) in self.ports_from.iter_mut() {
					if Checkbox( c_str!( "{}", port ), is_conn ) {
						*is_conn = if *is_conn {
							player.connect_from( port ).is_ok()
						}
						else {
							player.disconnect_from( port ).is_err()
						}
					}
				}
				EndPopup();
			}

			// ports to.
			SameLine( 0.0, -1.0 );
			if Button( c_str!( "Output to..." ), &ImVec2::zero() ) {
				OpenPopup( c_str!( "ports to" ) );
				self.ports_to = player.ports_to().unwrap_or_default();
			}
			if BeginPopup( c_str!( "ports to" ), 0 ) {
				for &mut (ref port, ref mut is_conn) in self.ports_to.iter_mut() {
					if Checkbox( c_str!( "{}", port ), is_conn ) {
						*is_conn = if *is_conn {
							player.connect_to( port ).is_ok()
						}
						else {
							player.disconnect_to( port ).is_err()
						}
					}
				}
				EndPopup();
			}

			SameLine( 0.0, -1.0 );
			if Button( c_str!( "Generate SMF" ), &ImVec2::zero() ) {
				self.events.push( Event::GenerateSmf );
			}
			SameLine( 0.0, -1.0 );
			if Button( c_str!( "\u{f021}" ), &ImVec2::zero() ) {
				self.events.push( Event::Refresh );
			}
		End();
		PopStyleVar( 1 );

		changed
	}
}
