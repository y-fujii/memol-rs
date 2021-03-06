// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::generator;
use crate::misc;
use crate::random;
use crate::ratio::Ratio;
use std::*;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub time: f64,
    pub prio: i8,
    pub msg: [u8; 3],
}

impl Event {
    pub fn new(time: f64, prio: i8, msg: &[u8]) -> Event {
        let mut this = Event {
            time: time,
            prio: prio,
            msg: [0; 3],
        };
        this.msg[..msg.len()].copy_from_slice(msg);
        this
    }

    pub fn len(&self) -> usize {
        // XXX
        3
    }

    pub fn validate(&self) -> bool {
        self.time.is_finite()
            && [0x80, 0x90, 0xb0, 0xe0].contains(&(self.msg[0] & 0xf0))
            && self.msg[1] & 0x80 == 0
            && self.msg[2] & 0x80 == 0
    }
}

pub struct Generator<'a> {
    rng: &'a random::Generator,
    events: Vec<Event>,
    timeline: Vec<f64>,
    bgn: i64,
    end: i64,
    tick: i64,
}

impl<'a> Generator<'a> {
    pub fn new(rng: &'a random::Generator, bgn: i64, end: i64, tick: i64) -> Self {
        Generator {
            rng: rng,
            events: Vec::new(),
            timeline: Vec::new(),
            bgn: bgn,
            end: end,
            tick: tick,
        }
    }

    pub fn add_score(
        &mut self,
        ch: usize,
        ir_score: &generator::ScoreIr,
        ir_vel: &generator::ValueIr,
        ir_ofs: &generator::ValueIr,
        ir_dur: &generator::ValueIr,
    ) {
        let mut evaluator = generator::Evaluator::new(self.rng);
        let mut offset = collections::HashMap::new();
        for f in ir_score.iter() {
            let nnum = match f.nnum {
                Some(v) => v,
                None => continue,
            };
            // accepts note off messages at end.
            if f.t0 < Ratio::new(self.bgn, self.tick) || Ratio::new(self.end, self.tick) < f.t1 {
                continue;
            }

            evaluator.set_note(ir_score, f);
            let dt = evaluator.eval(ir_dur, f.t0);
            let d0 = *offset
                .entry((f.t0, nnum))
                .or_insert_with(|| evaluator.eval(ir_ofs, f.t0));
            let d1 = *offset
                .entry((f.t1, nnum))
                .or_insert_with(|| evaluator.eval(ir_ofs, f.t1));

            // be careful of the numerical error which causes order inversion.
            let a = dt / evaluator.note_len;
            let t0 = f.t0.to_float() + d0;
            let t1 = (1.0 - a) * t0 + a * (f.t1.to_float() + d1);
            if t0 >= t1 {
                continue;
            }

            let vel = (evaluator.eval(ir_vel, f.t0) * 127.0).round().max(0.0).min(127.0);
            self.events
                .push(Event::new(t0, 1, &[(0x90 + ch) as u8, nnum as u8, vel as u8]));
            self.events
                .push(Event::new(t1, -1, &[(0x80 + ch) as u8, nnum as u8, vel as u8]));
        }
    }

    pub fn add_pitch(&mut self, ch: usize, ir: &generator::ValueIr) {
        let evaluator = generator::Evaluator::new(self.rng);
        let mut prev_v = 8192;
        for i in self.bgn..self.end {
            let t = Ratio::new(i, self.tick);
            let v = (evaluator.eval(ir, t) * 8192.0 + 8192.0).round().max(0.0).min(16383.0) as usize;
            if v != prev_v {
                let lsb = ((v >> 0) & 0x7f) as u8;
                let msb = ((v >> 7) & 0x7f) as u8;
                self.events
                    .push(Event::new(t.to_float(), 0, &[(0xe0 + ch) as u8, lsb, msb]));
                prev_v = v;
            }
        }
    }

    pub fn add_cc(&mut self, ch: usize, cc: usize, ir: &generator::ValueIr) {
        let evaluator = generator::Evaluator::new(self.rng);
        let mut prev_v = 255;
        for i in self.bgn..self.end {
            let t = Ratio::new(i, self.tick);
            let v = (evaluator.eval(ir, t) * 127.0).round().max(0.0).min(127.0) as u8;
            if v != prev_v {
                self.events
                    .push(Event::new(t.to_float(), 0, &[(0xb0 + ch) as u8, cc as u8, v]));
                prev_v = v;
            }
        }
    }

    pub fn add_tempo(&mut self, ir: &generator::ValueIr) {
        debug_assert!(self.timeline.len() == 0);
        let evaluator = generator::Evaluator::new(self.rng);
        // Kahan summation.
        let mut s = 0.0;
        let mut c = 0.0;
        for i in 0..self.end + 1 {
            self.timeline.push(s);
            let y = 1.0 / (self.tick as f64 * evaluator.eval(ir, Ratio::new(i, self.tick))) - c;
            let t = s + y;
            c = (t - s) - y;
            s = t;
        }
        self.timeline.push(s);
    }

    pub fn generate(mut self) -> Result<Vec<Event>, misc::Error> {
        self.events
            .sort_by(|x, y| (x.time, x.prio).partial_cmp(&(y.time, y.prio)).unwrap());
        if self.timeline.len() > 0 {
            for ev in self.events.iter_mut() {
                let i = (ev.time * self.tick as f64).floor() as usize;
                let i = cmp::min(cmp::max(i, 0), self.timeline.len() - 2);
                let f0 = self.timeline[i + 0];
                let f1 = self.timeline[i + 1];
                let a = ev.time * self.tick as f64 - i as f64;
                ev.time = (1.0 - a) * f0 + a * f1;
            }
        }
        Ok(self.events)
    }
}
