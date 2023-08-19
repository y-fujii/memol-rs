// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::chord;
use crate::random;

#[test]
fn test_random() {
    let rng = random::Generator::new();
    assert_eq!(rng.next_u64(), 10582614419484085930);
    assert_eq!(rng.next_u64(), 16147916016143995109);
    assert_eq!(rng.next_u64(), 5691192622506874316);
    assert_eq!(rng.next_u64(), 14606526736076162211);
    rng.jump();
    assert_eq!(rng.next_u64(), 4275479514889395181);
}

fn test_chord(text: &str, rhs: &[isize]) {
    let (pos, mut lhs) = chord::parse(text);
    assert_eq!(pos, text.len());
    let mut rhs = Vec::from(rhs);
    lhs.sort();
    rhs.sort();
    assert_eq!(lhs, rhs);
}

#[test]
fn test_chord_from_chordpro() {
    // <https://www.chordpro.org/chordpro/chordpro-chords/>.
    test_chord("C2", &[0, 2, 7]);
    test_chord("C3", &[0, 4]);
    test_chord("C4", &[0, 5, 7]);
    test_chord("C5", &[0, 7]);
    test_chord("C6", &[0, 4, 7, 9]);
    test_chord("C69", &[0, 4, 7, 9, 2]);
    test_chord("C7", &[0, 4, 7, 10]);
    test_chord("C7-5", &[0, 4, 6, 10]);
    test_chord("C7#5", &[0, 4, 8, 10]);
    test_chord("C7#9", &[0, 4, 7, 10, 3]);
    test_chord("C7#9#5", &[0, 4, 8, 10, 3]);
    test_chord("C7#9b5", &[0, 4, 6, 10, 3]);
    test_chord("C7#9#11", &[0, 7, 10, 3, 6]);
    test_chord("C7b5", &[0, 4, 6, 10]);
    test_chord("C7b9", &[0, 4, 7, 10, 1]);
    test_chord("C7b9#5", &[0, 4, 8, 10, 1]);
    test_chord("C7b9#9", &[0, 4, 7, 10, 1, 3]);
    test_chord("C7b9#11", &[0, 7, 10, 1, 6]);
    test_chord("C7b9b13", &[0, 4, 10, 1, 8]);
    test_chord("C7b9b5", &[0, 4, 6, 10, 1]);
    test_chord("C7b9sus", &[0, 5, 7, 10, 1]);
    test_chord("C7b13", &[0, 4, 10, 8]);
    test_chord("C7b13sus", &[0, 5, 10, 8]);
    test_chord("C7-9", &[0, 4, 7, 10, 1]);
    test_chord("C7-9#11", &[0, 7, 10, 1, 6]);
    test_chord("C7-9#5", &[0, 4, 8, 10, 1]);
    test_chord("C7-9#9", &[0, 4, 7, 10, 1, 3]);
    test_chord("C7-9-13", &[0, 4, 10, 1, 8]);
    test_chord("C7-9-5", &[0, 4, 6, 10, 1]);
    test_chord("C7-9sus", &[0, 5, 7, 10, 1]);
    test_chord("C711", &[0, 7, 10, 5]);
    test_chord("C7#11", &[0, 7, 10, 6]);
    test_chord("C7-13", &[0, 4, 10, 8]);
    test_chord("C7-13sus", &[0, 5, 10, 8]);
    test_chord("C7sus", &[0, 5, 7, 10]);
    test_chord("C7susadd3", &[0, 4, 5, 7, 10]);
    test_chord("C7+", &[0, 4, 8, 10]);
    test_chord("C7alt", &[0, 4, 10, 8]);
    test_chord("C9", &[0, 4, 7, 10, 2]);
    test_chord("C9+", &[0, 4, 8, 10, 2]);
    test_chord("C9#5", &[0, 4, 8, 10, 2]);
    test_chord("C9b5", &[0, 4, 6, 10, 2]);
    test_chord("C9-5", &[0, 4, 6, 10, 2]);
    test_chord("C9sus", &[0, 5, 7, 10, 2]);
    test_chord("C9add6", &[0, 4, 7, 9, 10, 2]);
    test_chord("Cmaj7", &[0, 4, 7, 11]);
    test_chord("Cmaj711", &[0, 7, 11, 5]);
    test_chord("Cmaj7#11", &[0, 7, 11, 6]);
    test_chord("Cmaj13", &[0, 11, 2, 5, 9]);
    test_chord("Cmaj7#5", &[0, 4, 8, 11]);
    test_chord("Cmaj7sus2", &[0, 2, 7, 11]);
    test_chord("Cmaj7sus4", &[0, 5, 7, 11]);
    test_chord("C^7", &[0, 4, 7, 11]);
    test_chord("C^711", &[0, 7, 11, 5]);
    test_chord("C^7#11", &[0, 7, 11, 6]);
    test_chord("C^7#5", &[0, 4, 8, 11]);
    test_chord("C^7sus2", &[0, 2, 7, 11]);
    test_chord("C^7sus4", &[0, 5, 7, 11]);
    test_chord("Cmaj9", &[0, 4, 7, 11, 2]);
    test_chord("Cmaj911", &[0, 7, 11, 2, 5]);
    test_chord("C^9", &[0, 4, 7, 11, 2]);
    test_chord("C^911", &[0, 7, 11, 2, 5]);
    test_chord("C^13", &[0, 11, 2, 5, 9]);
    test_chord("C^9#11", &[0, 7, 11, 2, 6]);
    test_chord("C11", &[0, 7, 10, 2, 5]);
    test_chord("C911", &[0, 7, 10, 2, 5]);
    test_chord("C9#11", &[0, 7, 10, 2, 6]);
    test_chord("C13", &[0, 10, 2, 5, 9]);
    test_chord("C13#11", &[0, 10, 2, 6, 9]);
    test_chord("C13#9", &[0, 10, 3, 5, 9]);
    test_chord("C13b9", &[0, 10, 1, 5, 9]);
    test_chord("Calt", &[0, 4, 8]);
    test_chord("Cadd2", &[0, 2, 4, 7]);
    test_chord("Cadd4", &[0, 4, 5, 7]);
    test_chord("Cadd9", &[0, 4, 7, 2]);
    test_chord("Csus2", &[0, 2, 7]);
    test_chord("Csus4", &[0, 5, 7]);
    test_chord("Csus9", &[0, 7, 10, 2]);
    test_chord("C6sus2", &[0, 2, 7, 9]);
    test_chord("C6sus4", &[0, 5, 7, 9]);
    test_chord("C7sus2", &[0, 2, 7, 10]);
    test_chord("C7sus4", &[0, 5, 7, 10]);
    test_chord("C13sus2", &[0, 10, 2, 5, 9]);
    test_chord("C13sus4", &[0, 10, 2, 5, 9]);
    test_chord("Cm#5", &[0, 3, 8]);
    test_chord("C-#5", &[0, 3, 8]);
    test_chord("Cm11", &[0, 7, 10, 2, 5]);
    test_chord("C-11", &[0, 7, 10, 2, 5]);
    test_chord("Cm6", &[0, 3, 7, 9]);
    test_chord("C-6", &[0, 3, 7, 9]);
    test_chord("Cm69", &[0, 3, 7, 9, 2]);
    test_chord("C-69", &[0, 3, 7, 9, 2]);
    test_chord("Cm7b5", &[0, 3, 6, 10]);
    test_chord("C-7b5", &[0, 3, 6, 10]);
    test_chord("Cm7-5", &[0, 3, 6, 10]);
    test_chord("C-7-5", &[0, 3, 6, 10]);
    test_chord("Cmmaj7", &[0, 3, 7, 11]);
    test_chord("C-maj7", &[0, 3, 7, 11]);
    test_chord("Cmmaj9", &[0, 3, 7, 11, 2]);
    test_chord("C-maj9", &[0, 3, 7, 11, 2]);
    test_chord("Cm9maj7", &[0, 3, 7, 11, 2]);
    test_chord("C-9maj7", &[0, 3, 7, 11, 2]);
    test_chord("Cm9^7", &[0, 3, 7, 11, 2]);
    test_chord("C-9^7", &[0, 3, 7, 11, 2]);
    test_chord("Cmadd9", &[0, 3, 7, 2]);
    test_chord("C-add9", &[0, 3, 7, 2]);
    test_chord("Cmb6", &[0, 3, 7, 8]);
    test_chord("C-b6", &[0, 3, 7, 8]);
    test_chord("Cm#7", &[0, 3, 7, 11]); // #7?
    test_chord("C-#7", &[0, 3, 7, 11]); // #7?
    test_chord("Cmsus4", &[0, 5, 7]);
    test_chord("Cmsus9", &[0, 7, 10, 2]);
    test_chord("C-sus4", &[0, 5, 7]);
    test_chord("C-sus9", &[0, 7, 10, 2]);
    test_chord("Cm7sus4", &[0, 5, 7, 10]);
    test_chord("C-7sus4", &[0, 5, 7, 10]);
    test_chord("Caug", &[0, 4, 8]);
    test_chord("C+", &[0, 4, 8]);
    test_chord("Cdim", &[0, 3, 6]);
    test_chord("C0", &[0, 3, 6]);
    test_chord("Cdim7", &[0, 3, 6, 9]);
    test_chord("Ch", &[0, 3, 6]);
    test_chord("Ch7", &[0, 3, 6, 10]);
    test_chord("Ch9", &[0, 3, 6, 10, 2]);
}

#[test]
fn test_chord_others() {
    test_chord("C", &[0, 4, 7]);
    test_chord("D", &[2, 6, 9]);
    test_chord("Db", &[1, 5, 8]);
    test_chord("D#", &[3, 7, 10]);
    test_chord("CM7(9, #11, 13)", &[0, 11, 2, 6, 9]);
    test_chord("C7(b9, #9, b13)", &[0, 4, 10, 1, 3, 8]);
    test_chord("C7(no3, omit5)", &[0, 10]);
    test_chord("C/D", &[2, 0, 4, 7]);
    test_chord("C/DM", &[2, 6, 9, 0, 4, 7]);
    test_chord("Cm3", &[0, 3]);
    test_chord("Cdim5", &[0, 6]);
    test_chord("Caug5", &[0, 8]);
    test_chord("Cma", &[0, 4, 7]);
    test_chord("Cmadd9", &[0, 3, 7, 2]);
    test_chord("Cmaadd9", &[0, 4, 7, 2]);
}

#[test]
fn test_voicing() {
    use crate::voicing;
    assert_eq!(voicing::voice_closed_with_center(&[0, 4, 7], 12), vec![7, 12, 16]);
    assert_eq!(
        voicing::voice_closed_with_center(&[0, 4, 7], 24),
        vec![7 + 12, 12 + 12, 16 + 12]
    );
    assert_eq!(voicing::voice_closed_with_center(&[0, 4, 7, 11], 7), vec![0, 4, 7, 11]);
    assert_eq!(
        voicing::voice_closed_with_center(&[0, 4, 7, 11], 9),
        vec![7, 11, 12, 16]
    );
}
