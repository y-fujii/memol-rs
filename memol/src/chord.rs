use std::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    First,
    Tension,
    Addition,
}

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
    n09: Option<isize>,
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
            n09: None,
            n09f: None,
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
            self.n09f, self.n09, self.n09s, self.n11, self.n13,
        ];
        for n in notes.iter() {
            if let Some(n) = *n {
                dst.push(root + n);
            }
        }
        dst
    }
}

fn parse_sign(s: &mut Stream) -> isize {
    if s.get_token("-") || s.get_token("b") {
        -1
    } else if s.get_token("+") || s.get_token("#") {
        1
    } else {
        0
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
        if !parse_tension(s, notes, Context::Addition) {
            s.pos = pos;
            return false;
        }
    } else {
        return false;
    }
    true
}

fn parse_tension(s: &mut Stream, notes: &mut Notes, ctx: Context) -> bool {
    let pos = s.pos;
    let sign = parse_sign(s);
    if s.get_token("13") {
        notes.n13 = Some(9 + sign);
        if ctx != Context::Addition {
            notes.n05 = None;
        }
        if ctx == Context::First {
            notes.n05 = None;
            notes.n07 = Some(notes.n07_candidate);
            notes.n09 = Some(2);
            notes.n11 = Some(5);
        }
    } else if s.get_token("11") {
        notes.n11 = Some(5 + sign);
        if ctx != Context::Addition {
            match sign {
                -1 | 0 => notes.n03 = None,
                1 => notes.n05 = None,
                _ => unreachable!(),
            }
        }
        if ctx == Context::First {
            notes.n07 = Some(notes.n07_candidate);
            notes.n09 = Some(2);
        }
    } else if s.get_token("9") {
        notes.n09 = None;
        match sign {
            -1 => notes.n09f = Some(1),
            0 => notes.n09 = Some(2),
            1 => notes.n09s = Some(3),
            _ => unreachable!(),
        }
        if ctx == Context::First {
            notes.n07 = Some(notes.n07_candidate);
        }
    } else if s.get_token("7") {
        // XXX: sign?
        notes.n07 = Some(notes.n07_candidate + sign);
    } else if s.get_token("6") {
        notes.n06 = Some(9 + sign);
    } else if s.get_token("5") {
        notes.n05 = Some(7 + sign);
        // XXX: dim5, aug5.
        if sign == 0 && ctx == Context::First {
            notes.n03 = None;
        }
    } else if s.get_token("4") {
        notes.n04 = Some(5 + sign);
        if ctx == Context::First {
            notes.n03 = None;
        }
    } else if s.get_token("3") {
        notes.n03 = Some(4 + sign);
        if ctx == Context::First {
            notes.n05 = None;
        }
    } else if s.get_token("2") {
        notes.n02 = Some(2 + sign);
        if ctx == Context::First {
            notes.n03 = None;
        }
    } else {
        s.pos = pos;
        return false;
    }
    true
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

    let mut ctx = Context::First;
    loop {
        if parse_symbol(s, notes) {
            continue;
        }
        if parse_tension(s, notes, ctx) {
            ctx = Context::Tension;
            continue;
        }
        if s.get_token("(") {
            ctx = Context::Tension;
            continue;
        }

        break;
    }

    // ToDo: no, omit, alt, dim5, aug5, on-chords, fractional chords.
    true
}

pub fn parse(text: &str) -> Vec<isize> {
    let mut s = Stream::new(text);
    let mut notes = Notes::new();
    parse_chord(&mut s, &mut notes);
    notes.notes()
}
