use std::*;

struct Stream<'a> {
    text: &'a str,
    pos: usize,
}

struct Tensions {
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

impl Tensions {
    fn new() -> Self {
        Tensions {
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

    fn notes_rev(&self, dst: &mut Vec<isize>, root: isize) {
        let tensions = [
            self.n13, self.n11, self.n09s, self.n09n, self.n09f, //
            self.n07, self.n06, self.n05, self.n04, self.n03, self.n02,
        ];
        for n in tensions.iter() {
            if let Some(n) = *n {
                dst.push(root + n);
            }
        }
    }
}

fn parse_note(s: &mut Stream) -> Option<isize> {
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
        return None;
    };

    // "C+" == "Caug" != "C#", "C-" == "Cdim" != "Cb".
    let sign = if s.get_token("b") {
        -1
    } else if s.get_token("#") {
        1
    } else {
        0
    };

    Some(note + sign)
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

fn parse_symbol(s: &mut Stream, tensions: &mut Tensions) -> bool {
    let pos = s.pos;
    if s.get_token("maj") || s.get_token("Maj") || s.get_token("M") || s.get_token("^") {
        tensions.n07_candidate = 11;
    } else if s.get_token("m") {
        tensions.n03 = Some(3);
    } else if s.get_token("dim") || s.get_token("0") {
        tensions.n03 = Some(3);
        tensions.n05 = Some(6);
        tensions.n07_candidate = 9;
    } else if s.get_token("aug") {
        tensions.n05 = Some(8);
    } else if s.get_token("h") {
        tensions.n03 = Some(3);
        tensions.n05 = Some(6);
    } else if s.get_token("sus2") {
        tensions.n03 = None;
        tensions.n02 = Some(2);
    } else if s.get_token("sus4") || s.get_token("sus") {
        tensions.n03 = None;
        tensions.n04 = Some(5);
    } else if s.get_token("add") {
        let Some(t) = parse_tension(s) else {
            s.pos = pos;
            return false;
        };
        add_tension_explicit(tensions, t);
    } else if s.get_token("omit") || s.get_token("no") {
        let Some(t) = parse_tension(s) else {
            s.pos = pos;
            return false;
        };
        omit_tension_explicit(tensions, t);
    } else {
        return false;
    }
    true
}

fn omit_tension_explicit(tensions: &mut Tensions, (note, sign): (isize, isize)) {
    match note {
        13 => tensions.n13 = None,
        11 => tensions.n11 = None,
        9 => match sign {
            -1 => tensions.n09f = None,
            0 => {
                tensions.n09f = None;
                tensions.n09n = None;
                tensions.n09s = None;
            }
            1 => tensions.n09s = None,
            _ => unreachable!(),
        },
        7 => tensions.n07 = None,
        6 => tensions.n06 = None,
        5 => tensions.n05 = None,
        4 => tensions.n04 = None,
        3 => tensions.n03 = None,
        2 => tensions.n02 = None,
        _ => (),
    }
}

fn omit_tension_implicit(tensions: &mut Tensions, t: (isize, isize)) {
    match t {
        (13, _) => tensions.n05 = None,
        (11, -1) => tensions.n03 = None,
        (11, 0) => tensions.n03 = None,
        (11, 1) => tensions.n05 = None,
        (5, 0) => tensions.n03 = None,
        (4, _) => tensions.n03 = None,
        (3, _) => tensions.n05 = None,
        (2, _) => tensions.n03 = None,
        _ => (),
    }
}

fn add_tension_explicit(tensions: &mut Tensions, (note, sign): (isize, isize)) {
    match note {
        13 => tensions.n13 = Some(9 + sign),
        11 => tensions.n11 = Some(5 + sign),
        9 => {
            tensions.n09n = None;
            match sign {
                -1 => tensions.n09f = Some(1),
                0 => tensions.n09n = Some(2),
                1 => tensions.n09s = Some(3),
                _ => unreachable!(),
            }
        }
        7 => tensions.n07 = Some(tensions.n07_candidate + sign),
        6 => tensions.n06 = Some(9 + sign),
        5 => tensions.n05 = Some(7 + sign),
        4 => tensions.n04 = Some(5 + sign),
        3 => tensions.n03 = Some(4 + sign),
        2 => tensions.n02 = Some(2 + sign),
        _ => (),
    }
}

fn add_tension_implicit(tensions: &mut Tensions, (note, _): (isize, isize)) {
    match note {
        13 => {
            tensions.n07 = Some(tensions.n07_candidate);
            tensions.n09n = Some(2);
            tensions.n11 = Some(5);
        }
        11 => {
            tensions.n07 = Some(tensions.n07_candidate);
            tensions.n09n = Some(2);
        }
        9 => {
            tensions.n07 = Some(tensions.n07_candidate);
        }
        _ => (),
    }
}

// ToDo: alt, dim5, aug5.
fn parse_elements(s: &mut Stream) -> Tensions {
    let mut tensions = Tensions::new();

    // "C-9" == "Cm9" != "C(-9)", "C+9" == "Caug9" != "C(+9)".
    if s.get_token("-") {
        tensions.n03 = Some(3);
    } else if s.get_token("+") {
        tensions.n05 = Some(8);
    };

    let mut is_first = true;
    loop {
        if parse_symbol(s, &mut tensions) {
            continue;
        }
        if let Some(t) = parse_tension(s) {
            add_tension_explicit(&mut tensions, t);
            omit_tension_implicit(&mut tensions, t);
            if mem::replace(&mut is_first, false) {
                add_tension_implicit(&mut tensions, t);
            }
            continue;
        }
        if s.get_token("(") {
            is_first = false;
            continue;
        }

        break;
    }

    tensions
}

pub fn parse(text: &str) -> (usize, Vec<isize>) {
    let mut s = Stream::new(text);
    let mut notes = Vec::new();

    let Some(root) = parse_note(&mut s) else {
        return (s.pos, Vec::new());
    };
    let t = parse_elements(&mut s);
    t.notes_rev(&mut notes, root);
    notes.push(root);

    loop {
        let pos = s.pos;
        if !s.get_token("on") && !s.get_token("/") {
            break;
        }
        let Some(root) = parse_note(&mut s) else {
            s.pos = pos;
            break;
        };
        let pos = s.pos;
        let t = parse_elements(&mut s);
        if s.pos > pos {
            // polychord.
            t.notes_rev(&mut notes, root);
        }
        notes.push(root);
    }

    notes.reverse();
    (s.pos, notes)
}
