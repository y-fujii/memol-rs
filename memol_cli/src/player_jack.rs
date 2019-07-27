// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::jack;
use crate::player;
use memol::{midi, misc};
use std::*;

const BUFFER_LEN: usize = 65536;

struct SharedData {
    //events: Option<Vec<midi::Event>>,
    events: Vec<midi::Event>,
    changed: bool,
    immediate_send: Vec<midi::Event>,
    immediate_recv: Vec<midi::Event>,
    exiting: bool,
}

struct LocalData {
    lib: sync::Arc<jack::Library>,
    jack: *mut jack::Client,
    port_send: *mut jack::Port,
    port_recv: *mut jack::Port,
    events: Vec<midi::Event>,
    immediate_send: Vec<midi::Event>,
    immediate_recv: Vec<midi::Event>,
    playing: bool,
    frame: u32,
    shared: sync::Arc<sync::Mutex<SharedData>>,
    condvar: sync::Arc<sync::Condvar>,
}

pub struct Player {
    lib: sync::Arc<jack::Library>,
    jack: *mut jack::Client,
    port_send: *mut jack::Port,
    port_recv: *mut jack::Port,
    _local: Box<LocalData>,
    shared: sync::Arc<sync::Mutex<SharedData>>,
    condvar: sync::Arc<sync::Condvar>,
    callback_thread: Option<thread::JoinHandle<()>>,
}

unsafe impl Send for Player {}

impl Drop for Player {
    fn drop(&mut self) {
        self.exit_callback_thread();
        unsafe {
            (self.lib.client_close)(self.jack);
        }
    }
}

impl player::Player for Player {
    fn on_received_boxed(&mut self, f: Box<dyn 'static + Fn(&[midi::Event]) + Send>) {
        self.exit_callback_thread();
        self.callback_thread = Some(thread::spawn({
            let shared = self.shared.clone();
            let condvar = self.condvar.clone();
            move || Self::callback_proc(f, shared, condvar)
        }));
    }

    fn set_data(&mut self, events: &[midi::Event]) {
        let mut shared = self.shared.lock().unwrap();
        shared.events = events.to_vec();
        shared.changed = true;
    }

    fn ports_from(&self) -> io::Result<Vec<(String, bool)>> {
        self.ports(jack::PORT_IS_OUTPUT)
    }

    fn connect_from(&self, port: &str) -> io::Result<()> {
        unsafe { self.connect(format!("{}\0", port).as_ptr(), (self.lib.port_name)(self.port_recv)) }
    }

    fn disconnect_from(&self, port: &str) -> io::Result<()> {
        unsafe { self.disconnect(format!("{}\0", port).as_ptr(), (self.lib.port_name)(self.port_recv)) }
    }

    fn ports_to(&self) -> io::Result<Vec<(String, bool)>> {
        self.ports(jack::PORT_IS_INPUT)
    }

    fn connect_to(&self, port: &str) -> io::Result<()> {
        unsafe { self.connect((self.lib.port_name)(self.port_send), format!("{}\0", port).as_ptr()) }
    }

    fn disconnect_to(&self, port: &str) -> io::Result<()> {
        unsafe { self.disconnect((self.lib.port_name)(self.port_send), format!("{}\0", port).as_ptr()) }
    }

    fn send(&self, evs: &[midi::Event]) {
        let mut shared = self.shared.lock().unwrap();
        shared.immediate_send.extend_from_slice(evs);
    }

    fn play(&self) {
        unsafe {
            (self.lib.transport_start)(self.jack);
        }
    }

    fn stop(&self) {
        unsafe {
            (self.lib.transport_stop)(self.jack);
        }
    }

    fn seek(&self, time: f64) {
        debug_assert!(time >= 0.0);
        unsafe {
            let mut pos: jack::Position = mem::uninitialized();
            (self.lib.transport_query)(self.jack, &mut pos);
            (self.lib.transport_locate)(self.jack, (time * pos.frame_rate as f64).round() as u32);
        }
    }

    fn status(&self) -> (bool, f64) {
        unsafe {
            let mut pos: jack::Position = mem::uninitialized();
            let playing = match (self.lib.transport_query)(self.jack, &mut pos) {
                jack::TransportState::Stopped => false,
                _ => true,
            };
            // the resolution of jack_position_t::frame is per process cycles.
            // jack_get_current_transport_frame() estimates the current
            // position more accurately.
            let frame = (self.lib.get_current_transport_frame)(self.jack);
            let loc = frame as f64 / pos.frame_rate as f64;
            (playing, loc)
        }
    }

    fn info(&self) -> String {
        "JACK is running.".into()
    }
}

impl Player {
    pub fn new(name: &str) -> io::Result<Self> {
        unsafe {
            let lib = sync::Arc::new(jack::Library::new()?);

            let jack = (lib.client_open)(format!("{}\0", name).as_ptr(), 0, ptr::null_mut());
            if jack.is_null() {
                return Self::error("jack_client_open().");
            }
            let port_send =
                (lib.port_register)(jack, "out\0".as_ptr(), jack::DEFAULT_MIDI_TYPE, jack::PORT_IS_OUTPUT, 0);
            if port_send.is_null() {
                (lib.client_close)(jack);
                return Self::error("jack_port_register().");
            }
            let port_recv = (lib.port_register)(jack, "in\0".as_ptr(), jack::DEFAULT_MIDI_TYPE, jack::PORT_IS_INPUT, 0);
            if port_recv.is_null() {
                (lib.client_close)(jack);
                return Self::error("jack_port_register().");
            }

            let condvar = sync::Arc::new(sync::Condvar::new());
            let shared = sync::Arc::new(sync::Mutex::new(SharedData {
                events: Vec::new(),
                changed: false,
                immediate_send: Vec::new(),
                immediate_recv: Vec::with_capacity(BUFFER_LEN),
                exiting: false,
            }));

            let local = Box::new(LocalData {
                lib: lib.clone(),
                jack: jack,
                port_send: port_send,
                port_recv: port_recv,
                events: Vec::new(),
                immediate_send: Vec::with_capacity(BUFFER_LEN),
                immediate_recv: Vec::with_capacity(BUFFER_LEN),
                playing: false,
                frame: 0,
                shared: shared.clone(),
                condvar: condvar.clone(),
            });

            if (lib.set_process_callback)(jack, Player::process_callback, &*local) != 0 {
                return Self::error("jack_set_process_callback().");
            }
            if (lib.activate)(jack) != 0 {
                return Self::error("jack_activate().");
            }

            Ok(Player {
                lib: lib,
                jack: jack,
                port_send: port_send,
                port_recv: port_recv,
                _local: local,
                shared: shared,
                condvar: condvar,
                callback_thread: None,
            })
        }
    }

    fn ports(&self, port_type: usize) -> io::Result<Vec<(String, bool)>> {
        unsafe {
            let self_port = match port_type {
                jack::PORT_IS_INPUT => self.port_send,
                jack::PORT_IS_OUTPUT => self.port_recv,
                _ => panic!(),
            };

            let c_result = (self.lib.get_ports)(self.jack, ptr::null(), jack::DEFAULT_MIDI_TYPE, port_type);
            if c_result.is_null() {
                return Self::error("jack_get_ports().");
            }
            let mut r_result = Vec::new();
            let mut it = c_result;
            while !(*it).is_null() {
                match ffi::CStr::from_ptr(*it as *const _).to_str() {
                    Ok(v) => {
                        let is_conn = (self.lib.port_connected_to)(self_port, *it);
                        r_result.push((v.into(), is_conn != 0));
                    }
                    Err(_) => {
                        (self.lib.free)(c_result);
                        return Self::error("jack_get_ports().");
                    }
                }
                it = it.offset(1);
            }
            (self.lib.free)(c_result);
            Ok(r_result)
        }
    }

    unsafe fn connect(&self, from: *const u8, to: *const u8) -> io::Result<()> {
        if (self.lib.connect)(self.jack, from, to) != 0 {
            return Self::error("jack_connect().");
        }
        Ok(())
    }

    unsafe fn disconnect(&self, from: *const u8, to: *const u8) -> io::Result<()> {
        if (self.lib.disconnect)(self.jack, from, to) != 0 {
            return Self::error("jack_disconnect().");
        }
        Ok(())
    }

    fn error<T>(text: &str) -> io::Result<T> {
        Err(io::Error::new(io::ErrorKind::Other, text))
    }

    // avoid freeing memory in this function.
    extern "C" fn process_callback(size: u32, local: *const dyn any::Any) -> i32 {
        unsafe {
            let local = &mut *(local as *mut LocalData);

            let buf_send = (local.lib.port_get_buffer)(local.port_send, size);
            let buf_recv = (local.lib.port_get_buffer)(local.port_recv, size);
            (local.lib.midi_clear_buffer)(buf_send);

            let mut pos: jack::Position = mem::uninitialized();
            let state = (local.lib.transport_query)(local.jack, &mut pos);
            let playing = state == jack::TransportState::Rolling;

            // receive midi data.
            for i in 0.. {
                let mut ev = mem::uninitialized();
                if (local.lib.midi_event_get)(&mut ev, buf_recv, i) != 0 {
                    break;
                }
                let msg = slice::from_raw_parts(ev.buffer, ev.size);
                let prio = match msg[0] & 0xf0 {
                    0x80 => -1,
                    0x90 => 1,
                    0xb0 => 0,
                    _ => continue,
                };
                local
                    .immediate_recv
                    .push(midi::Event::new(ev.time as f64 / pos.frame_rate as f64, prio, msg));
            }

            // sync local <=> shared.
            let mut changed = false;
            if let Ok(mut shared) = local.shared.try_lock() {
                if mem::replace(&mut shared.changed, false) {
                    mem::swap(&mut local.events, &mut shared.events);
                    changed = true;
                }

                shared.immediate_recv.extend(local.immediate_recv.drain(..));
                local.immediate_send.extend(shared.immediate_send.drain(..));
                if shared.immediate_recv.len() > 0 {
                    local.condvar.notify_one();
                }
            }

            // send an all-sound-off event.
            if changed || local.playing != playing || (playing && local.frame != pos.frame) {
                for ch in 0..16 {
                    let msg: [u8; 3] = [0xb0 + ch, 0x78, 0x00]; // all sound off.
                    (local.lib.midi_event_write)(buf_send, 0, msg.as_ptr(), msg.len());
                    let msg: [u8; 3] = [0xb0 + ch, 0x79, 0x00]; // reset all controllers.
                    (local.lib.midi_event_write)(buf_send, 0, msg.as_ptr(), msg.len());
                }
            }

            // send immediate data.
            for ev in local.immediate_send.drain(..) {
                // XXX: add delay.
                (local.lib.midi_event_write)(buf_send, 0, ev.msg.as_ptr(), ev.len());
            }

            // send playing data.
            if playing {
                let frame = |ev: &midi::Event| (ev.time * pos.frame_rate as f64).round() as isize - pos.frame as isize;
                let ibgn = misc::bsearch_boundary(&local.events, |ev| frame(ev) < 0);
                let iend = misc::bsearch_boundary(&local.events, |ev| frame(ev) < size as isize);
                for ev in local.events[ibgn..iend].iter() {
                    (local.lib.midi_event_write)(buf_send, frame(ev) as u32, ev.msg.as_ptr(), ev.len());
                }

                if ibgn == local.events.len() {
                    (local.lib.transport_stop)(local.jack);
                }
            }

            local.playing = playing;
            local.frame = pos.frame + if playing { size } else { 0 };
        }
        0
    }

    fn exit_callback_thread(&mut self) {
        if let Some(thread) = self.callback_thread.take() {
            self.shared.lock().unwrap().exiting = true;
            self.condvar.notify_all();
            thread.join().unwrap();
        }
    }

    fn callback_proc(
        on_received: Box<dyn 'static + Fn(&[midi::Event]) + Send>,
        shared: sync::Arc<sync::Mutex<SharedData>>,
        condvar: sync::Arc<sync::Condvar>,
    ) {
        let mut evs = Vec::with_capacity(BUFFER_LEN);
        loop {
            {
                let mut shared = shared.lock().unwrap();
                while !shared.exiting && shared.immediate_recv.len() == 0 {
                    shared = condvar.wait(shared).unwrap();
                }
                if shared.exiting {
                    break;
                }
                mem::swap(&mut evs, &mut shared.immediate_recv);
            }
            on_received(&evs);
            evs.clear();
        }
    }
}
