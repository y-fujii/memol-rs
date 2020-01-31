// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::player;
use memol::midi;
use std::io::Write;
use std::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum CtsMessage {
    Status(bool, f64),
    Immediate(Vec<midi::Event>),
}

impl CtsMessage {
    pub fn deserialize_from<T: io::Read>(reader: T) -> bincode::Result<Self> {
        let msg: CtsMessage = bincode::config().limit(1 << 24).deserialize_from(reader)?;
        let is_valid = match &msg {
            CtsMessage::Status(_, loc) => loc.is_finite(),
            CtsMessage::Immediate(evs) => evs.iter().all(|e| e.validate()),
        };
        if is_valid {
            Ok(msg)
        } else {
            Err(Box::new(bincode::ErrorKind::Custom("invalid data.".into())))
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum StcMessage {
    Data(Vec<midi::Event>),
    Immediate(Vec<midi::Event>),
}

impl StcMessage {
    pub fn deserialize_from<T: io::Read>(reader: T) -> bincode::Result<Self> {
        let msg: StcMessage = bincode::config().limit(1 << 24).deserialize_from(reader)?;
        let is_valid = match &msg {
            StcMessage::Data(evs) => evs.iter().all(|e| e.validate()),
            StcMessage::Immediate(evs) => evs.iter().all(|e| e.validate()),
        };
        if is_valid {
            Ok(msg)
        } else {
            Err(Box::new(bincode::ErrorKind::Custom("invalid data.".into())))
        }
    }
}

struct CtsShared {
    on_received: Box<dyn 'static + Fn(&[midi::Event]) + Send>,
    playing: bool,
    location: f64,
    arrival: time::Instant,
}

struct StcShared {
    events: Vec<midi::Event>,
    senders: Vec<sync::mpsc::Sender<Vec<u8>>>,
}

pub struct Player {
    listener: net::TcpListener,
    thread: Option<thread::JoinHandle<()>>,
    cts_shared: sync::Arc<sync::Mutex<CtsShared>>,
    stc_shared: sync::Arc<sync::Mutex<StcShared>>,
}

impl Drop for Player {
    fn drop(&mut self) {
        #[cfg(target_family = "unix")]
        {
            use os::unix::io::AsRawFd;
            let fd = self.listener.as_raw_fd();
            unsafe { libc::shutdown(fd, libc::SHUT_RDWR) };
            self.thread.take().unwrap().join().ok();
        }
        // other OSs throw the threads away.
    }
}

impl player::Player for Player {
    fn on_received_boxed(&mut self, f: Box<dyn 'static + Fn(&[midi::Event]) + Send>) {
        let mut shared = self.cts_shared.lock().unwrap();
        shared.on_received = f;
    }

    fn set_data(&mut self, events: &[midi::Event]) {
        let msg = bincode::serialize(&StcMessage::Data(events.to_vec())).unwrap();
        let mut shared = self.stc_shared.lock().unwrap();
        shared.events = events.to_vec();
        shared.senders.retain(|s| s.send(msg.clone()).is_ok());
    }

    fn ports_from(&self) -> io::Result<Vec<(String, bool)>> {
        Err(io::ErrorKind::Other.into())
    }

    fn connect_from(&self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn disconnect_from(&self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn ports_to(&self) -> io::Result<Vec<(String, bool)>> {
        Err(io::ErrorKind::Other.into())
    }

    fn connect_to(&self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn disconnect_to(&self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn send(&self, events: &[midi::Event]) {
        let msg = bincode::serialize(&StcMessage::Immediate(events.into())).unwrap();
        let mut shared = self.stc_shared.lock().unwrap();
        shared.senders.retain(|s| s.send(msg.clone()).is_ok());
    }

    fn play(&self) {}

    fn stop(&self) {}

    fn seek(&self, _: f64) {}

    fn status(&self) -> (bool, f64) {
        let shared = self.cts_shared.lock().unwrap();
        let delta = shared.arrival.elapsed();
        let loc = if shared.playing {
            1e-9 * delta.subsec_nanos() as f64 + delta.as_secs() as f64 + shared.location
        } else {
            shared.location
        };
        (shared.playing, loc)
    }

    fn info(&self) -> String {
        let cnt = self.stc_shared.lock().unwrap().senders.len();
        format!("{} VST plugin(s) are connected.", cnt)
    }
}

impl Player {
    pub fn new<T: net::ToSocketAddrs>(addr: T) -> io::Result<Self> {
        let cts_shared = sync::Arc::new(sync::Mutex::new(CtsShared {
            on_received: Box::new(|_| ()),
            playing: false,
            location: 0.0,
            arrival: time::Instant::now(),
        }));
        let stc_shared = sync::Arc::new(sync::Mutex::new(StcShared {
            events: Vec::new(),
            senders: Vec::new(),
        }));

        let listener = net::TcpListener::bind(addr)?;
        let thread = thread::spawn({
            let listener = listener.try_clone()?;
            let cts_shared = cts_shared.clone();
            let stc_shared = stc_shared.clone();
            move || {
                for stream in listener.incoming() {
                    let stream = match stream {
                        Ok(s) => s,
                        Err(e) => {
                            if e.kind() == io::ErrorKind::InvalidInput {
                                break;
                            } else {
                                continue;
                            }
                        }
                    };
                    stream.set_nodelay(true).ok();

                    // reader thread.
                    thread::spawn({
                        let shared = cts_shared.clone();
                        let stream = match stream.try_clone() {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        move || {
                            let mut stream = io::BufReader::new(stream);
                            while let Ok(msg) = CtsMessage::deserialize_from(&mut stream) {
                                match msg {
                                    CtsMessage::Status(playing, location) => {
                                        let now = time::Instant::now();
                                        let mut shared = shared.lock().unwrap();
                                        shared.playing = playing;
                                        shared.location = location;
                                        shared.arrival = now;
                                    }
                                    CtsMessage::Immediate(evs) => {
                                        let shared = shared.lock().unwrap();
                                        // XXX
                                        (shared.on_received)(&evs);
                                    }
                                }
                            }
                            stream.get_ref().shutdown(net::Shutdown::Both).ok();
                        }
                    });

                    // writer thread.
                    thread::spawn({
                        let shared = stc_shared.clone();
                        let mut stream = match stream.try_clone() {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        move || {
                            || -> Result<(), Box<dyn error::Error>> {
                                let (sender, receiver) = sync::mpsc::channel();
                                let events = {
                                    let mut shared = shared.lock().unwrap();
                                    shared.senders.push(sender);
                                    shared.events.clone()
                                };
                                stream.write_all(&bincode::serialize(&StcMessage::Data(events)).unwrap())?;
                                loop {
                                    let msg = receiver.recv()?;
                                    stream.write_all(&msg)?;
                                }
                            }()
                            .ok();
                            stream.shutdown(net::Shutdown::Both).ok();
                        }
                    });
                }
            }
        });

        Ok(Player {
            listener: listener,
            thread: Some(thread),
            cts_shared: cts_shared,
            stc_shared: stc_shared,
        })
    }
}
