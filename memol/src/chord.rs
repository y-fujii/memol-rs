// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

struct Stream<'a> {
    text: &'a str,
    pos: usize,
}

struct Tensions {
    ns: [Option<isize>; 7],
    n1: [bool; 3],
    n6_candidate: isize,
}

impl<'a> Stream<'a> {
    fn new(text: &'a str) -> Self {
        Stream { text: text, pos: 0 }
    }

    fn skip_ws(&mut self) {
        for (i, c) in self.text[self.pos..].char_indices() {
            if !c.is_whitespace() && c != ',' {
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
            ns: [None; 7],
            n1: [false; 3],
            n6_candidate: 0,
        }
    }

    fn add(&mut self, t: (usize, isize)) {
        match (t.0 % 7, t.1) {
            (1, -1) => {
                self.n1[0] = true;
                self.n1[1] = false;
            }
            (1, 0) => self.n1 = [false, true, false],
            (1, 1) => {
                self.n1[1] = false;
                self.n1[2] = true;
            }
            (6, 0) => self.ns[6] = Some(self.n6_candidate),
            (n, s) => self.ns[n] = Some(s),
        }
    }

    fn omit(&mut self, t: (usize, isize)) {
        match (t.0 % 7, t.1) {
            (1, -1) => self.n1[0] = false,
            (1, 0) => self.n1.fill(false),
            (1, 1) => self.n1[2] = false,
            (n, _) => self.ns[n] = None,
        }
    }

    fn omit_by(&mut self, t: (usize, isize)) {
        match t.0 {
            10 => self.ns[2] = None,
            12 => self.ns[4] = None,
            _ => (),
        }
    }

    fn get_notes_rev(&self, dst: &mut Vec<isize>, root: isize) {
        let mixolydian = [0, 2, 4, 5, 7, 9, 10];
        for i in (2..7).rev() {
            if let Some(s) = self.ns[i] {
                dst.push(root + mixolydian[i] + s);
            }
        }

        let offsets = [1, 2, 3];
        for i in (0..3).rev() {
            if self.n1[i] {
                dst.push(root + offsets[i]);
            }
        }
    }
}

fn parse_note(stream: &mut Stream) -> Option<isize> {
    let note = if stream.get_token("C") {
        0
    } else if stream.get_token("D") {
        2
    } else if stream.get_token("E") {
        4
    } else if stream.get_token("F") {
        5
    } else if stream.get_token("G") {
        7
    } else if stream.get_token("A") {
        9
    } else if stream.get_token("B") {
        11
    } else {
        return None;
    };

    // "C+" == "Caug" != "C#", "C-" == "Cm" != "Cb".
    let sign = if stream.get_token("b") {
        -1
    } else if stream.get_token("#") {
        1
    } else {
        0
    };

    Some(note + sign)
}

fn parse_tension(stream: &mut Stream) -> Option<(usize, isize)> {
    let pos = stream.pos;

    let sign = if stream.get_token("-") || stream.get_token("b") {
        -1
    } else if stream.get_token("+") || stream.get_token("#") {
        1
    } else {
        0
    };

    // returns zero-based index.
    let note = if stream.get_token("13") {
        12
    } else if stream.get_token("11") {
        10
    } else if stream.get_token("9") {
        8
    } else if stream.get_token("7") {
        6
    } else if stream.get_token("6") {
        5
    } else if stream.get_token("5") {
        4
    } else if stream.get_token("4") {
        3
    } else if stream.get_token("3") {
        2
    } else if stream.get_token("2") {
        1
    } else {
        stream.pos = pos;
        return None;
    };

    Some((note, sign))
}

fn parse_symbol(stream: &mut Stream, tensions: &mut Tensions) -> bool {
    let pos = stream.pos;
    if stream.get_token("maj") || stream.get_token("Maj") || stream.get_token("M") || stream.get_token("^") {
        tensions.n6_candidate = 1;
    } else if stream.get_token("m") || stream.get_token("-") {
        tensions.ns[2] = Some(-1);
    } else if stream.get_token("dim") || stream.get_token("0") {
        tensions.ns[2] = Some(-1);
        tensions.ns[4] = Some(-1);
        tensions.n6_candidate = -1;
    } else if stream.get_token("aug") || stream.get_token("+") {
        tensions.ns[4] = Some(1);
    } else if stream.get_token("h") {
        tensions.ns[2] = Some(-1);
        tensions.ns[4] = Some(-1);
    } else if stream.get_token("sus") {
        // XXX
        let pos = stream.pos;
        if let None = parse_tension(stream) {
            tensions.ns[3] = Some(0);
        }
        stream.pos = pos;
        tensions.ns[2] = None;
    } else if stream.get_token("add") {
        let Some(t) = parse_tension(stream) else {
            stream.pos = pos;
            return false;
        };
        tensions.add(t);
    } else if stream.get_token("omit") || stream.get_token("no") {
        let Some(t) = parse_tension(stream) else {
            stream.pos = pos;
            return false;
        };
        tensions.omit(t);
    } else if stream.get_token("alt") {
        tensions.n1[1] = false;
        tensions.ns[4] = None;
        tensions.ns[5] = Some(-1);
    } else {
        return false;
    }
    true
}

fn set_tension_first(tensions: &mut Tensions, t: (usize, isize)) {
    match t {
        (1, _) => tensions.ns[2] = None,
        (2, 0) => tensions.ns[4] = None,
        (3, _) => tensions.ns[2] = None,
        (4, 0) => tensions.ns[2] = None,
        _ => (),
    }
    if t != (2, 0) && t != (4, 0) {
        tensions.add(t);
        tensions.omit_by(t);
    }
    for i in [6, 8, 10] {
        if i >= t.0 {
            break;
        }
        tensions.add((i, 0));
        tensions.omit_by((i, 0));
    }
}

fn parse_elements(stream: &mut Stream) -> Tensions {
    let mut tensions = Tensions::new();
    tensions.ns[2] = Some(0);
    tensions.ns[4] = Some(0);

    // "C-9" == "Cm9" != "C(-9)", "C+9" == "Caug9" != "C(+9)".
    parse_symbol(stream, &mut tensions);

    let mut is_first = true;
    let mut nest_level = 0;
    loop {
        if let Some(t) = parse_tension(stream) {
            if mem::replace(&mut is_first, false) {
                set_tension_first(&mut tensions, t);
            } else {
                tensions.add(t);
                tensions.omit_by(t);
            }
            continue;
        }
        if parse_symbol(stream, &mut tensions) {
            continue;
        }
        if stream.get_token("(") {
            is_first = false;
            nest_level += 1;
            continue;
        }
        if nest_level > 0 && stream.get_token(")") {
            nest_level -= 1;
            continue;
        }

        break;
    }

    tensions
}

pub fn parse(text: &str) -> (usize, Vec<isize>) {
    let mut stream = Stream::new(text);
    let mut notes = Vec::new();

    let Some(root) = parse_note(&mut stream) else {
        return (stream.pos, Vec::new());
    };
    let t = parse_elements(&mut stream);
    t.get_notes_rev(&mut notes, root);
    notes.push(root);

    loop {
        let pos = stream.pos;
        if !stream.get_token("on") && !stream.get_token("/") {
            break;
        }
        let Some(root) = parse_note(&mut stream) else {
            stream.pos = pos;
            break;
        };
        let pos = stream.pos;
        let t = parse_elements(&mut stream);
        if stream.pos > pos {
            // polychord.
            t.get_notes_rev(&mut notes, root);
        }
        notes.push(root);
    }

    notes.reverse();
    (stream.pos, notes)
}
