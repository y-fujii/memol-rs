#![allow( dead_code )]
#![allow( improper_ctypes )]

use std::*;
use std::os::raw::c_ulong;


pub const PORT_IS_OUTPUT: c_ulong = 2;

#[repr( C )]
pub enum TransportState {
	Stopped,
	Rolling,
	Looping,
	Starting,
	NetStarting,
}

pub enum Client     {}
pub enum Port       {}
pub enum PortBuffer {}

#[repr( C )]
pub struct Position {
	pub unique_1:         u64,
	pub usecs:            u64,
	pub frame_rate:       u32,
	pub frame:            u32,
	pub valid:            u32,
	pub bar:              i32,
	pub beat:             i32,
	pub tick:             i32,
	pub bar_start_tick:   f64,
	pub beats_per_bar:    f32,
	pub beat_type:        f32,
	pub ticks_per_beat:   f64,
	pub beats_per_minute: f64,
	pub frame_time:       f64,
	pub next_time:        f64,
	pub bbt_offset:       u32,
	pub padding:          [i32; 9],
	pub unique_2:         u64,
}

pub type ProcessCallback = extern "C" fn( u32, *mut any::Any ) -> i32;
pub type SyncCallback    = extern "C" fn( TransportState, *mut Position, *mut any::Any ) -> i32;

extern "C" {
	pub fn jack_activate( _: *mut Client ) -> i32;
	pub fn jack_client_close( _: *mut Client ) -> i32;
	pub fn jack_client_open( _: *const i8, _: u32, _: *mut u32, ... ) -> *mut Client;
	pub fn jack_connect( _: *mut Client, _: *const i8, _: *const i8 ) -> i32;
	pub fn jack_midi_clear_buffer( _: *mut PortBuffer ) -> ();
	pub fn jack_midi_event_write( _: *mut PortBuffer, _: u32, _: *const u8, _: usize ) -> i32;
	pub fn jack_port_get_buffer( _: *mut Port, _: u32 ) -> *mut PortBuffer;
	pub fn jack_port_name( _: *const Port ) -> *const i8;
	pub fn jack_port_register( _: *mut Client, _: *const i8, _: *const i8, _: c_ulong, _: c_ulong ) -> *mut Port;
	pub fn jack_set_process_callback( _: *mut Client, _: ProcessCallback, _: *mut any::Any ) -> i32;
	pub fn jack_set_sync_callback( _: *mut Client, _: SyncCallback, _: *mut any::Any ) -> i32;
	pub fn jack_transport_locate( _: *mut Client, _: u32 ) -> i32;
	pub fn jack_transport_query( _: *const Client, _: *mut Position ) -> TransportState;
	pub fn jack_transport_start( _: *mut Client ) -> ();
}
