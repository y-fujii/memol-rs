// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::*;
use memol::midi;
use std::*;

pub struct Player {
    location: cell::Cell<f64>,
}

impl player::Player for Player {
    fn on_received_boxed(&mut self, _: Box<dyn 'static + Fn(&[midi::Event]) + Send>) {}

    fn set_data(&mut self, _: &[midi::Event]) {}

    fn ports_from(&mut self) -> io::Result<Vec<(String, bool)>> {
        Err(io::ErrorKind::Other.into())
    }

    fn connect_from(&mut self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn disconnect_from(&mut self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn ports_to(&mut self) -> io::Result<Vec<(String, bool)>> {
        Err(io::ErrorKind::Other.into())
    }

    fn connect_to(&mut self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn disconnect_to(&mut self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn send(&mut self, _: &[midi::Event]) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn play(&mut self) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn stop(&mut self) {}

    fn seek(&mut self, loc: f64) {
        self.location.set(loc);
    }

    fn status(&mut self) -> (bool, f64) {
        (false, self.location.get())
    }

    fn info(&mut self) -> String {
        String::new()
    }
}

impl Player {
    pub fn new() -> Player {
        Player {
            location: cell::Cell::new(0.0),
        }
    }
}
