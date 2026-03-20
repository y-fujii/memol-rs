// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use memol::midi;
use std::*;

pub trait Player: Send {
    fn on_received_boxed(&mut self, _: Box<dyn 'static + Fn(&[midi::Event]) + Send>);
    fn set_data(&mut self, _: &[midi::Event]);
    fn ports_from(&mut self) -> io::Result<Vec<(String, bool)>>;
    fn connect_from(&mut self, _: &str) -> io::Result<()>;
    fn disconnect_from(&mut self, _: &str) -> io::Result<()>;
    fn ports_to(&mut self) -> io::Result<Vec<(String, bool)>>;
    fn connect_to(&mut self, _: &str) -> io::Result<()>;
    fn disconnect_to(&mut self, _: &str) -> io::Result<()>;
    fn send(&mut self, _: &[midi::Event]) -> io::Result<()>;
    fn play(&mut self) -> io::Result<()>;
    fn stop(&mut self);
    fn seek(&mut self, _: f64);
    fn status(&mut self) -> (bool, f64);
    fn info(&mut self) -> String;
}

pub trait PlayerExt {
    fn on_received<T: 'static + Fn(&[midi::Event]) + Send>(&mut self, _: T);
}

impl<T: Player> PlayerExt for T {
    fn on_received<U: 'static + Fn(&[midi::Event]) + Send>(&mut self, f: U) {
        self.on_received_boxed(Box::new(f));
    }
}

impl PlayerExt for &mut dyn Player {
    fn on_received<T: 'static + Fn(&[midi::Event]) + Send>(&mut self, f: T) {
        self.on_received_boxed(Box::new(f));
    }
}

impl PlayerExt for Box<dyn Player> {
    fn on_received<T: 'static + Fn(&[midi::Event]) + Send>(&mut self, f: T) {
        self.on_received_boxed(Box::new(f));
    }
}
