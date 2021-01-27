use std::*;

pub const PORT_IS_INPUT: usize = 1;
pub const PORT_IS_OUTPUT: usize = 2;
pub const DEFAULT_MIDI_TYPE: *const u8 = b"8 bit raw midi\0" as *const _;

#[repr(C)]
pub struct MidiEvent {
    pub time: u32,
    pub size: usize,
    pub buffer: *mut u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum TransportState {
    Stopped,
    Rolling,
    Looping,
    Starting,
    NetStarting,
}

pub enum Client {}
pub enum Port {}
pub enum PortBuffer {}

#[repr(C)]
pub struct Position {
    pub unique_1: u64,
    pub usecs: u64,
    pub frame_rate: u32,
    pub frame: u32,
    pub valid: u32,
    pub bar: i32,
    pub beat: i32,
    pub tick: i32,
    pub bar_start_tick: f64,
    pub beats_per_bar: f32,
    pub beat_type: f32,
    pub ticks_per_beat: f64,
    pub beats_per_minute: f64,
    pub frame_time: f64,
    pub next_time: f64,
    pub bbt_offset: u32,
    pub padding: [i32; 9],
    pub unique_2: u64,
}

pub type ProcessCallback = extern "C" fn(u32, *mut ffi::c_void) -> i32;

pub struct Library {
    _lib: libloading::Library,
    pub activate: unsafe extern "C" fn(*mut Client) -> i32,
    pub client_close: unsafe extern "C" fn(*mut Client) -> i32,
    pub client_open: unsafe extern "C" fn(*const u8, u32, *mut u32, ...) -> *mut Client,
    pub connect: unsafe extern "C" fn(*mut Client, *const u8, *const u8) -> i32,
    pub disconnect: unsafe extern "C" fn(*mut Client, *const u8, *const u8) -> i32,
    pub free: unsafe extern "C" fn(*const dyn any::Any) -> (),
    pub get_current_transport_frame: unsafe extern "C" fn(*const Client) -> u32,
    pub get_ports: unsafe extern "C" fn(*mut Client, *const u8, *const u8, usize) -> *const *const u8,
    pub midi_clear_buffer: unsafe extern "C" fn(*mut PortBuffer) -> (),
    pub midi_event_get: unsafe extern "C" fn(*mut MidiEvent, *mut PortBuffer, u32) -> i32,
    pub midi_event_write: unsafe extern "C" fn(*mut PortBuffer, u32, *const u8, usize) -> i32,
    pub port_connected_to: unsafe extern "C" fn(*const Port, *const u8) -> i32,
    pub port_get_buffer: unsafe extern "C" fn(*mut Port, u32) -> *mut PortBuffer,
    pub port_name: unsafe extern "C" fn(*const Port) -> *const u8,
    pub port_register: unsafe extern "C" fn(*mut Client, *const u8, *const u8, usize, usize) -> *mut Port,
    pub set_process_callback: unsafe extern "C" fn(*mut Client, ProcessCallback, *mut ffi::c_void) -> i32,
    pub transport_locate: unsafe extern "C" fn(*mut Client, u32) -> i32,
    pub transport_query: unsafe extern "C" fn(*const Client, *mut Position) -> TransportState,
    pub transport_start: unsafe extern "C" fn(*mut Client) -> (),
    pub transport_stop: unsafe extern "C" fn(*mut Client) -> (),
}

impl Library {
    pub fn new() -> Result<Self, libloading::Error> {
        #[cfg(all(target_family = "unix", not(target_os = "macos")))]
        let path = "libjack.so.0";
        #[cfg(all(target_family = "unix", target_os = "macos"))]
        let path = "libjack.dylib";
        #[cfg(all(target_family = "windows", target_pointer_width = "64"))]
        let path = "libjack64.dll";
        #[cfg(all(target_family = "windows", target_pointer_width = "32"))]
        let path = "libjack.dll";

        unsafe {
            let lib = libloading::Library::new(path)?;
            Ok(Self {
                activate: *lib.get(b"jack_activate\0")?,
                client_close: *lib.get(b"jack_client_close\0")?,
                client_open: *lib.get(b"jack_client_open\0")?,
                connect: *lib.get(b"jack_connect\0")?,
                disconnect: *lib.get(b"jack_disconnect\0")?,
                free: *lib.get(b"jack_free\0")?,
                get_current_transport_frame: *lib.get(b"jack_get_current_transport_frame\0")?,
                get_ports: *lib.get(b"jack_get_ports\0")?,
                midi_clear_buffer: *lib.get(b"jack_midi_clear_buffer\0")?,
                midi_event_get: *lib.get(b"jack_midi_event_get\0")?,
                midi_event_write: *lib.get(b"jack_midi_event_write\0")?,
                port_connected_to: *lib.get(b"jack_port_connected_to\0")?,
                port_get_buffer: *lib.get(b"jack_port_get_buffer\0")?,
                port_name: *lib.get(b"jack_port_name\0")?,
                port_register: *lib.get(b"jack_port_register\0")?,
                set_process_callback: *lib.get(b"jack_set_process_callback\0")?,
                transport_locate: *lib.get(b"jack_transport_locate\0")?,
                transport_query: *lib.get(b"jack_transport_query\0")?,
                transport_start: *lib.get(b"jack_transport_start\0")?,
                transport_stop: *lib.get(b"jack_transport_stop\0")?,
                _lib: lib,
            })
        }
    }
}
