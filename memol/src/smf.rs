// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use crate::midi;


fn delta_time( buf: &mut Vec<u8>, t: u32 ) {
	debug_assert!( t < 1 << 28 );
	for i in [ 21, 14, 7u32 ].iter() {
		if t >> i != 0 {
			buf.push( ((t >> i) & 0x7f | 0x80) as u8 );
		}
	}
	buf.push( (t & 0x7f) as u8 );
}

pub fn generate_smf( events: &[midi::Event], unit: u16 ) -> Vec<u8> {
	let mut buf = Vec::new();
	buf.extend( b"MThd" );
	buf.extend( &6u32.to_be_bytes() ); // chunk length.
	buf.extend( &0u16.to_be_bytes() ); // format type.
	buf.extend( &1u16.to_be_bytes() ); // # of tracks.
	buf.extend( &unit.to_be_bytes() );
	buf.extend( b"MTrk" );
	let idx_len = buf.len();
	buf.extend( &0u32.to_be_bytes() );
	let idx_bgn = buf.len();

	let mut t = 0.0;
	for ev in events.iter() {
		// XXX: assumes 120 beat/min.
		let dt = (2.0 * unit as f64) * (ev.time - t);
		delta_time( &mut buf, dt.round() as u32 );
		buf.extend( &ev.msg[.. ev.len()] );
		t = ev.time;
	}
	delta_time( &mut buf, 0 );
	buf.extend( &[ 0xff, 0x2f, 0x00 ] );

	let idx_end = buf.len();
	let len = idx_end - idx_bgn;
	buf[idx_len .. idx_bgn].copy_from_slice( &(len as u32).to_be_bytes() );
	buf
}
