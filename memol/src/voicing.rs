// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

pub fn voice_closed_with_center(notes: &[isize], center: isize) -> Vec<isize> {
    assert!(notes.len() > 1);

    let mut sorted = Vec::new();
    for n in notes.iter() {
        sorted.push(n.rem_euclid(12));
    }
    sorted.sort();
    for i in 0..notes.len() - 1 {
        sorted.push(sorted[i] + 12);
    }

    let mut best_d2 = isize::MAX;
    let mut best_lo = 0;
    let mut best_oct = 0;
    for lo in 0..notes.len() {
        let hi = lo + notes.len() - 1;
        let t = 2 * center - (sorted[lo] + sorted[hi]);
        let oct = (t + 12).div_euclid(24);
        let d2 = (t - 24 * oct).abs();

        // penalty for too narrow interval on top/bottom notes.
        let d2 = d2 + cmp::max(24 * 3 - 24 * (sorted[hi] - sorted[hi - 1]), 0);
        let d2 = d2 + cmp::max(2 * 3 - 2 * (sorted[lo + 1] - sorted[lo]), 0);

        if d2 <= best_d2 {
            best_d2 = d2;
            best_lo = lo;
            best_oct = oct;
        }
    }

    let mut dst = Vec::new();
    for i in 0..notes.len() {
        dst.push(sorted[best_lo + i] + 12 * best_oct);
    }

    dst
}

pub fn split_bass_and_chord<'a>(notes: &'a [isize], n_min: usize) -> (isize, &'a [isize]) {
    assert!(notes.len() > 0);

    if notes.len() < n_min {
        (notes[0], &notes[0..])
    } else {
        (notes[0], &notes[1..])
    }
}
