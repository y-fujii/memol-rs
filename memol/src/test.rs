#[test]
fn test_random() {
    use crate::random;
    let rng = random::Generator::new();
    assert_eq!(rng.next_u64(), 10582614419484085930);
    assert_eq!(rng.next_u64(), 16147916016143995109);
    assert_eq!(rng.next_u64(), 5691192622506874316);
    assert_eq!(rng.next_u64(), 14606526736076162211);
    rng.jump();
    assert_eq!(rng.next_u64(), 4275479514889395181);
}

#[test]
fn test_chord() {
    use crate::chord::parse;

    // <https://www.chordpro.org/chordpro/chordpro-chords/>.
    assert_eq!(parse("C2"), vec![0, 2, 7]);
    assert_eq!(parse("C3"), vec![0, 4]);
    assert_eq!(parse("C4"), vec![0, 5, 7]);
    assert_eq!(parse("C5"), vec![0, 7]);
    assert_eq!(parse("C6"), vec![0, 4, 7, 9]);
    assert_eq!(parse("C69"), vec![0, 4, 7, 9, 2]);
    assert_eq!(parse("C7"), vec![0, 4, 7, 10]);
    assert_eq!(parse("C7-5"), vec![0, 4, 6, 10]);
    assert_eq!(parse("C7#5"), vec![0, 4, 8, 10]);
    assert_eq!(parse("C7#9"), vec![0, 4, 7, 10, 3]);
    assert_eq!(parse("C7#9#5"), vec![0, 4, 8, 10, 3]);
    assert_eq!(parse("C7#9b5"), vec![0, 4, 6, 10, 3]);
    assert_eq!(parse("C7#9#11"), vec![0, 4, 10, 3, 6]);
    assert_eq!(parse("C7b5"), vec![0, 4, 6, 10]);
    assert_eq!(parse("C7b9"), vec![0, 4, 7, 10, 1]);
    assert_eq!(parse("C7b9#5"), vec![0, 4, 8, 10, 1]);
    assert_eq!(parse("C7b9#9"), vec![0, 4, 7, 10, 1, 3]);
    assert_eq!(parse("C7b9#11"), vec![0, 4, 10, 1, 6]);
    assert_eq!(parse("C7b9b13"), vec![0, 4, 10, 1, 8]);
    assert_eq!(parse("C7b9b5"), vec![0, 4, 6, 10, 1]);
    assert_eq!(parse("C7b9sus"), vec![0, 5, 7, 10, 1]);
    assert_eq!(parse("C7b13"), vec![0, 4, 10, 8]);
    assert_eq!(parse("C7b13sus"), vec![0, 5, 10, 8]);
    assert_eq!(parse("C7-9"), vec![0, 4, 7, 10, 1]);
    assert_eq!(parse("C7-9#11"), vec![0, 4, 10, 1, 6]);
    /*
    assert_eq!(parse("C7-9#5"), vec![]);
    assert_eq!(parse("C7-9#9"), vec![]);
    assert_eq!(parse("C7-9-13"), vec![]);
    assert_eq!(parse("C7-9-5"), vec![]);
    assert_eq!(parse("C7-9sus"), vec![]);
    assert_eq!(parse("C711"), vec![]);
    assert_eq!(parse("C7#11"), vec![]);
    assert_eq!(parse("C7-13"), vec![]);
    assert_eq!(parse("C7-13sus"), vec![]);
    assert_eq!(parse("C7sus"), vec![]);
    assert_eq!(parse("C7susadd3"), vec![]);
    assert_eq!(parse("C7+"), vec![]);
    assert_eq!(parse("C7alt"), vec![]);
    assert_eq!(parse("C9"), vec![]);
    assert_eq!(parse("C9+"), vec![]);
    assert_eq!(parse("C9#5"), vec![]);
    assert_eq!(parse("C9b5"), vec![]);
    assert_eq!(parse("C9-5"), vec![]);
    assert_eq!(parse("C9sus"), vec![]);
    assert_eq!(parse("C9add6"), vec![]);
    assert_eq!(parse("Cmaj7"), vec![]);
    assert_eq!(parse("Cmaj711"), vec![]);
    assert_eq!(parse("Cmaj7#11"), vec![]);
    assert_eq!(parse("Cmaj13"), vec![]);
    assert_eq!(parse("Cmaj7#5"), vec![]);
    assert_eq!(parse("Cmaj7sus2"), vec![]);
    assert_eq!(parse("Cmaj7sus4"), vec![]);
    assert_eq!(parse("C^7"), vec![]);
    assert_eq!(parse("C^711"), vec![]);
    assert_eq!(parse("C^7#11"), vec![]);
    assert_eq!(parse("C^7#5"), vec![]);
    assert_eq!(parse("C^7sus2"), vec![]);
    assert_eq!(parse("C^7sus4"), vec![]);
    assert_eq!(parse("Cmaj9"), vec![]);
    assert_eq!(parse("Cmaj911"), vec![]);
    assert_eq!(parse("C^9"), vec![]);
    assert_eq!(parse("C^911"), vec![]);
    assert_eq!(parse("C^13"), vec![]);
    assert_eq!(parse("C^9#11"), vec![]);
    assert_eq!(parse("C11"), vec![]);
    assert_eq!(parse("C911"), vec![]);
    assert_eq!(parse("C9#11"), vec![]);
    assert_eq!(parse("C13"), vec![]);
    assert_eq!(parse("C13#11"), vec![]);
    assert_eq!(parse("C13#9"), vec![]);
    assert_eq!(parse("C13b9"), vec![]);
    assert_eq!(parse("Calt"), vec![]);
    assert_eq!(parse("Cadd2"), vec![]);
    assert_eq!(parse("Cadd4"), vec![]);
    assert_eq!(parse("Cadd9"), vec![]);
    assert_eq!(parse("Csus2"), vec![]);
    assert_eq!(parse("Csus4"), vec![]);
    assert_eq!(parse("Csus9"), vec![]);
    assert_eq!(parse("C6sus2"), vec![]);
    assert_eq!(parse("C6sus4"), vec![]);
    assert_eq!(parse("C7sus2"), vec![]);
    assert_eq!(parse("C7sus4"), vec![]);
    assert_eq!(parse("C13sus2"), vec![]);
    assert_eq!(parse("C13sus4"), vec![]);
    assert_eq!(parse("Cm#5"), vec![]);
    assert_eq!(parse("C-#5"), vec![]);
    assert_eq!(parse("Cm11"), vec![]);
    assert_eq!(parse("C-11"), vec![]);
    assert_eq!(parse("Cm6"), vec![]);
    assert_eq!(parse("C-6"), vec![]);
    assert_eq!(parse("Cm69"), vec![]);
    assert_eq!(parse("C-69"), vec![]);
    assert_eq!(parse("Cm7b5"), vec![]);
    assert_eq!(parse("C-7b5"), vec![]);
    assert_eq!(parse("Cm7-5"), vec![]);
    assert_eq!(parse("C-7-5"), vec![]);
    assert_eq!(parse("Cmmaj7"), vec![]);
    assert_eq!(parse("C-maj7"), vec![]);
    assert_eq!(parse("Cmmaj9"), vec![]);
    assert_eq!(parse("C-maj9"), vec![]);
    assert_eq!(parse("Cm9maj7"), vec![]);
    assert_eq!(parse("C-9maj7"), vec![]);
    assert_eq!(parse("Cm9^7"), vec![]);
    assert_eq!(parse("C-9^7"), vec![]);
    assert_eq!(parse("Cmadd9"), vec![]);
    assert_eq!(parse("C-add9"), vec![]);
    assert_eq!(parse("Cmb6"), vec![]);
    assert_eq!(parse("C-b6"), vec![]);
    assert_eq!(parse("Cm#7"), vec![]);
    assert_eq!(parse("C-#7"), vec![]);
    assert_eq!(parse("Cmsus4"), vec![]);
    assert_eq!(parse("Cmsus9"), vec![]);
    assert_eq!(parse("C-sus4"), vec![]);
    assert_eq!(parse("C-sus9"), vec![]);
    assert_eq!(parse("Cm7sus4"), vec![]);
    assert_eq!(parse("C-7sus4"), vec![]);
    assert_eq!(parse("Caug"), vec![]);
    assert_eq!(parse("C+"), vec![]);
    assert_eq!(parse("Cdim"), vec![]);
    assert_eq!(parse("C0"), vec![]);
    assert_eq!(parse("Cdim7"), vec![]);
    assert_eq!(parse("Ch"), vec![]);
    assert_eq!(parse("Ch7"), vec![]);
    assert_eq!(parse("Ch9"), vec![]);
    */

    assert_eq!(parse("C"), vec![0, 4, 7]);
    assert_eq!(parse("D"), vec![2, 6, 9]);
    assert_eq!(parse("Db"), vec![1, 5, 8]);
    assert_eq!(parse("D#"), vec![3, 7, 10]);
    assert_eq!(parse("Cm"), vec![0, 3, 7]);
    assert_eq!(parse("CM"), vec![0, 4, 7]);
    assert_eq!(parse("C7"), vec![0, 4, 7, 10]);
    assert_eq!(parse("Cm7"), vec![0, 3, 7, 10]);
    assert_eq!(parse("CM7"), vec![0, 4, 7, 11]);
    assert_eq!(parse("CmM7"), vec![0, 3, 7, 11]);
    assert_eq!(parse("Cdim"), vec![0, 3, 6]);
    assert_eq!(parse("Caug"), vec![0, 4, 8]);
    assert_eq!(parse("Cdim7"), vec![0, 3, 6, 9]);
    assert_eq!(parse("Caug7"), vec![0, 4, 8, 10]);
    assert_eq!(parse("CaugM7"), vec![0, 4, 8, 11]);
    assert_eq!(parse("C9"), vec![0, 4, 7, 10, 2]);
    assert_eq!(parse("C69"), vec![0, 4, 7, 9, 2]);
    assert_eq!(parse("Cm9"), vec![0, 3, 7, 10, 2]);
    assert_eq!(parse("CM9"), vec![0, 4, 7, 11, 2]);
    assert_eq!(parse("CmM9"), vec![0, 3, 7, 11, 2]);
    assert_eq!(parse("CM79"), vec![0, 4, 7, 11, 2]);
    assert_eq!(parse("CM7(9)"), vec![0, 4, 7, 11, 2]);
    assert_eq!(parse("CM7(9, #11, 13)"), vec![0, 4, 11, 2, 6, 9]);
    assert_eq!(parse("Cadd9"), vec![0, 4, 7, 2]);
    assert_eq!(parse("Cmadd9"), vec![0, 3, 7, 2]);
    assert_eq!(parse("C7add9"), vec![0, 4, 7, 10, 2]);
    assert_eq!(parse("CmM7add9"), vec![0, 3, 7, 11, 2]);
    assert_eq!(parse("Csus4"), vec![0, 5, 7]);
    assert_eq!(parse("C7sus4"), vec![0, 5, 7, 10]);
    assert_eq!(parse("Csus47"), vec![0, 5, 7, 10]);
    assert_eq!(parse("C9sus4(#11)"), vec![0, 5, 10, 2, 6]);
    assert_eq!(parse("C13sus4add9"), vec![0, 5, 10, 2, 5, 9]);
    assert_eq!(parse("C13sus4add9(#11)"), vec![0, 5, 10, 2, 6, 9]);
    assert_eq!(parse("Cm7b5"), vec![0, 3, 6, 10]);
    assert_eq!(parse("C7(b9, #9, b13)"), vec![0, 4, 10, 1, 3, 8]);
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
