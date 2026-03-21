// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::midi;
use std::*;

fn smf_delta_time(buf: &mut Vec<u8>, t: u32) {
    debug_assert!(t < 1 << 28);
    for i in [21, 14, 7u32].iter() {
        if t >> i != 0 {
            buf.push(((t >> i) & 0x7f | 0x80) as u8);
        }
    }
    buf.push((t & 0x7f) as u8);
}

pub fn smf_generate(events: &[midi::Event], unit: u16) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend(b"MThd");
    buf.extend(&6u32.to_be_bytes()); // chunk length.
    buf.extend(&0u16.to_be_bytes()); // format type.
    buf.extend(&1u16.to_be_bytes()); // # of tracks.
    buf.extend(&unit.to_be_bytes());
    buf.extend(b"MTrk");
    let idx_len = buf.len();
    buf.extend(&0u32.to_be_bytes());
    let idx_bgn = buf.len();

    let mut t = 0.0;
    for ev in events.iter() {
        // XXX: assumes 120 beat/min.
        let dt = (2.0 * unit as f64) * (ev.time - t);
        smf_delta_time(&mut buf, dt.round() as u32);
        buf.extend(&ev.msg[..ev.len()]);
        t = ev.time;
    }
    smf_delta_time(&mut buf, 0);
    buf.extend(&[0xff, 0x2f, 0x00]);

    let idx_end = buf.len();
    let len = idx_end - idx_bgn;
    buf[idx_len..idx_bgn].copy_from_slice(&(len as u32).to_be_bytes());
    buf
}

fn time_code_bytes(t: f64) -> [u8; 4] {
    let floor = t.floor();
    let ff = (30.0 * (t - floor)).round();
    let ss = floor as u64 % 60;
    let mm = floor as u64 / 60 % 60;
    let hh = floor as u64 / 3600;
    [ff as u8, ss as u8, mm as u8, 0b0110_0000 | hh as u8]
}

pub fn time_code_full(t: f64) -> [u8; 10] {
    let b = time_code_bytes(t);
    [0xf0, 0x7f, 0x7f, 0x01, 0x01, b[3], b[2], b[1], b[0], 0xf7]
}

pub fn time_code_add(dst: &mut Vec<midi::Event>, t_min: f64, t_max: f64) {
    let i0 = (15.0 * t_min).floor() as usize;
    let i1 = (15.0 * t_max).ceil() as usize;
    for i in i0..i1 {
        let packet = u32::from_le_bytes(time_code_bytes(i as f64 / 15.0));
        for j in 0..8 {
            let t = (8 * i + j) as f64 / 120.0;
            if t < t_min || t_max <= t {
                continue;
            }
            let piece = 0xf & (packet >> (4 * j));
            dst.push(midi::Event::new(t, -1, &[0xf1, (j << 4) as u8 | piece as u8]));
        }
    }
}
