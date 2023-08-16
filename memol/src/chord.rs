use std::*;

struct Stream<'a> {
    text: &'a str,
    pos: usize,
}

struct Notes {
    root: Option<isize>,
    n02: Option<isize>,
    n03: Option<isize>,
    n04: Option<isize>,
    n05: Option<isize>,
    n06: Option<isize>,
    n07_candidate: isize,
    n07: Option<isize>,
    n09f: Option<isize>,
    n09n: Option<isize>,
    n09s: Option<isize>,
    n11: Option<isize>,
    n13: Option<isize>,
}

impl<'a> Stream<'a> {
    fn new(text: &'a str) -> Self {
        Stream { text: text, pos: 0 }
    }

    fn skip_ws(&mut self) {
        for (i, c) in self.text[self.pos..].char_indices() {
            if !c.is_whitespace() && c != ',' && c != ')' {
                self.pos += i;
                return;
            }
        }
        self.pos = self.text.len();
    }

    fn get_token(&mut self, tok: &str) -> bool {
        self.skip_ws();
        if self.text[self.pos..].starts_with(tok) {
            self.pos += tok.len();
            true
        } else {
            false
        }
    }
}

impl Notes {
    fn new() -> Self {
        Notes {
            root: None,
            n02: None,
            n03: Some(4),
            n04: None,
            n05: Some(7),
            n06: None,
            n07_candidate: 10,
            n07: None,
            n09f: None,
            n09n: None,
            n09s: None,
            n11: None,
            n13: None,
        }
    }

    fn notes(&self) -> Vec<isize> {
        let Some(root) = self.root else {
            return Vec::new();
        };

        let mut dst = Vec::new();
        dst.push(root);
        let notes = [
            self.n02, self.n03, self.n04, self.n05, self.n06, self.n07, //
            self.n09f, self.n09n, self.n09s, self.n11, self.n13,
        ];
        for n in notes.iter() {
            if let Some(n) = *n {
                dst.push(root + n);
            }
        }
        dst
    }
}

fn parse_note(s: &mut Stream, notes: &mut Notes) -> bool {
    let note = if s.get_token("C") {
        0
    } else if s.get_token("D") {
        2
    } else if s.get_token("E") {
        4
    } else if s.get_token("F") {
        5
    } else if s.get_token("G") {
        7
    } else if s.get_token("A") {
        9
    } else if s.get_token("B") {
        11
    } else {
        return false;
    };

    // "C+" == "Caug", "C-" == "Cdim".
    let sign = if s.get_token("b") {
        -1
    } else if s.get_token("#") {
        1
    } else {
        0
    };

    notes.root = Some(note + sign);
    true
}

fn parse_tension(s: &mut Stream) -> Option<(isize, isize)> {
    let pos = s.pos;

    let sign = if s.get_token("-") || s.get_token("b") {
        -1
    } else if s.get_token("+") || s.get_token("#") {
        1
    } else {
        0
    };

    let note = if s.get_token("13") {
        13
    } else if s.get_token("11") {
        11
    } else if s.get_token("9") {
        9
    } else if s.get_token("7") {
        7
    } else if s.get_token("6") {
        6
    } else if s.get_token("5") {
        5
    } else if s.get_token("4") {
        4
    } else if s.get_token("3") {
        3
    } else if s.get_token("2") {
        2
    } else {
        s.pos = pos;
        return None;
    };

    Some((note, sign))
}

fn parse_symbol(s: &mut Stream, notes: &mut Notes) -> bool {
    let pos = s.pos;
    if s.get_token("maj") || s.get_token("Maj") || s.get_token("M") || s.get_token("^") {
        notes.n07_candidate = 11;
    } else if s.get_token("m") {
        notes.n03 = Some(3);
    } else if s.get_token("dim") || s.get_token("0") {
        notes.n03 = Some(3);
        notes.n05 = Some(6);
        notes.n07_candidate = 9;
    } else if s.get_token("aug") {
        notes.n05 = Some(8);
    } else if s.get_token("h") {
        notes.n03 = Some(3);
        notes.n05 = Some(6);
    } else if s.get_token("sus2") {
        notes.n03 = None;
        notes.n02 = Some(2);
    } else if s.get_token("sus4") || s.get_token("sus") {
        notes.n03 = None;
        notes.n04 = Some(5);
    } else if s.get_token("add") {
        let Some(tension) = parse_tension(s) else {
            s.pos = pos;
            return false;
        };
        add_tension_explicit(notes, tension);
    } else if s.get_token("omit") || s.get_token("no") {
        let Some(tension) = parse_tension(s) else {
            s.pos = pos;
            return false;
        };
        omit_tension_explicit(notes, tension);
    } else {
        return false;
    }
    true
}

fn omit_tension_explicit(notes: &mut Notes, (note, sign): (isize, isize)) {
    match note {
        13 => notes.n13 = None,
        11 => notes.n11 = None,
        9 => match sign {
            -1 => notes.n09f = None,
            0 => {
                // XXX
                notes.n09f = None;
                notes.n09n = None;
                notes.n09s = None;
            }
            1 => notes.n09s = None,
            _ => unreachable!(),
        },
        7 => notes.n07 = None,
        6 => notes.n06 = None,
        5 => notes.n05 = None,
        4 => notes.n04 = None,
        3 => notes.n03 = None,
        2 => notes.n02 = None,
        _ => (),
    }
}

fn omit_tension_implicit(notes: &mut Notes, tension: (isize, isize)) {
    match tension {
        (13, _) => notes.n05 = None,
        (11, -1) => notes.n03 = None,
        (11, 0) => notes.n03 = None,
        (11, 1) => notes.n05 = None,
        (5, 0) => notes.n03 = None,
        (4, _) => notes.n03 = None,
        (3, _) => notes.n05 = None,
        (2, _) => notes.n03 = None,
        _ => (),
    }
}

fn add_tension_explicit(notes: &mut Notes, (note, sign): (isize, isize)) {
    match note {
        13 => notes.n13 = Some(9 + sign),
        11 => notes.n11 = Some(5 + sign),
        9 => {
            notes.n09n = None; // XXX
            match sign {
                -1 => notes.n09f = Some(1),
                0 => notes.n09n = Some(2),
                1 => notes.n09s = Some(3),
                _ => unreachable!(),
            }
        }
        7 => notes.n07 = Some(notes.n07_candidate + sign),
        6 => notes.n06 = Some(9 + sign),
        5 => notes.n05 = Some(7 + sign),
        4 => notes.n04 = Some(5 + sign),
        3 => notes.n03 = Some(4 + sign),
        2 => notes.n02 = Some(2 + sign),
        _ => (),
    }
}

fn add_tension_implicit(notes: &mut Notes, (note, _): (isize, isize)) {
    match note {
        13 => {
            notes.n07 = Some(notes.n07_candidate);
            notes.n09n = Some(2);
            notes.n11 = Some(5);
        }
        11 => {
            notes.n07 = Some(notes.n07_candidate);
            notes.n09n = Some(2);
        }
        9 => {
            notes.n07 = Some(notes.n07_candidate);
        }
        _ => (),
    }
}

fn parse_chord(s: &mut Stream, notes: &mut Notes) -> bool {
    if !parse_note(s, notes) {
        return false;
    }

    // "C-9" == "Cm9" != "C(-9)", "C+9" == "Caug9" != "C(+9)".
    if s.get_token("-") {
        notes.n03 = Some(3);
    } else if s.get_token("+") {
        notes.n05 = Some(8);
    };

    let mut is_first = true;
    loop {
        if parse_symbol(s, notes) {
            continue;
        }
        if let Some(tension) = parse_tension(s) {
            add_tension_explicit(notes, tension);
            omit_tension_implicit(notes, tension);
            if is_first {
                add_tension_implicit(notes, tension);
                is_first = false;
            }
            continue;
        }
        if s.get_token("(") {
            is_first = false;
            continue;
        }

        break;
    }

    // ToDo: alt, dim5, aug5, on-chords, fractional chords.
    true
}

pub fn parse(text: &str) -> Vec<isize> {
    let mut s = Stream::new(text);
    let mut notes = Notes::new();
    parse_chord(&mut s, &mut notes);
    notes.notes()
}
