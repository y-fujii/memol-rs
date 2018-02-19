// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use midi;


fn delta_time( buf: &mut Vec<u8>, t: u32 ) {
	debug_assert!( t < 1 << 28 );
	for i in [ 21, 14, 7u32 ].iter() {
		if t >> i != 0 {
			buf.push( ((t >> i) & 0x7f | 0x80) as u8 );
		}
	}
	buf.push( (t & 0x7f) as u8 );
}

pub fn write_smf( buf: &mut io::Write, events: &[midi::Event], unit: u16 ) -> io::Result<()> {
	let mut content = Vec::new();
	let mut t = 0.0;
	for ev in events.iter() {
		// XXX: assumes 120 beat/min.
		let dt = (2.0 * unit as f64) * (ev.time - t);
		delta_time( &mut content, dt.round() as u32 );
		content.extend( &ev.msg[.. ev.len as usize] );
		t = ev.time;
	}
	delta_time( &mut content, 0 );
	content.extend( &[ 0xff, 0x2f, 0x00 ] );

	buf.write_all( b"MThd" )?;
	buf.write_all( &misc::u32_to_bytes_be( 6 ) )?; // chunk length.
	buf.write_all( &misc::u16_to_bytes_be( 0 ) )?; // format type.
	buf.write_all( &misc::u16_to_bytes_be( 1 ) )?; // # of tracks.
	buf.write_all( &misc::u16_to_bytes_be( unit ) )?;
	buf.write_all( b"MTrk" )?;
	buf.write_all( &misc::u32_to_bytes_be( content.len() as u32 ) )?;
	buf.write_all( &content )?;

	Ok( () )
}
