// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

struct Stream<'a> {
    text: &'a str,
    pos: usize,
}

struct Tensions {
    n03: Option<isize>,
    n05: Option<isize>,
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
            n03: Some(4),
            n05: Some(7),
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
            self.n07, self.n13, self.n05, self.n11, self.n03, self.n09s, self.n09n, self.n09f,
        ];
        for n in tensions.iter() {
            if let Some(n) = *n {
                dst.push(root + n);
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

    // "C+" == "Caug" != "C#", "C-" == "Cdim" != "Cb".
    let sign = if stream.get_token("b") {
        -1
    } else if stream.get_token("#") {
        1
    } else {
        0
    };

    Some(note + sign)
}

fn parse_tension(stream: &mut Stream) -> Option<(isize, isize)> {
    let pos = stream.pos;

    let sign = if stream.get_token("-") || stream.get_token("b") {
        -1
    } else if stream.get_token("+") || stream.get_token("#") {
        1
    } else {
        0
    };

    let note = if stream.get_token("13") {
        13
    } else if stream.get_token("11") {
        11
    } else if stream.get_token("9") {
        9
    } else if stream.get_token("7") {
        7
    } else if stream.get_token("6") {
        6
    } else if stream.get_token("5") {
        5
    } else if stream.get_token("4") {
        4
    } else if stream.get_token("3") {
        3
    } else if stream.get_token("2") {
        2
    } else {
        stream.pos = pos;
        return None;
    };

    Some((note, sign))
}

fn parse_symbol(stream: &mut Stream, tensions: &mut Tensions) -> bool {
    let pos = stream.pos;
    if stream.get_token("maj") || stream.get_token("Maj") || stream.get_token("M") || stream.get_token("^") {
        tensions.n07_candidate = 11;
    } else if stream.get_token("m") || stream.get_token("-") {
        tensions.n03 = Some(3);
    } else if stream.get_token("dim") || stream.get_token("0") {
        tensions.n03 = Some(3);
        tensions.n05 = Some(6);
        tensions.n07_candidate = 9;
    } else if stream.get_token("aug") || stream.get_token("+") {
        tensions.n05 = Some(8);
    } else if stream.get_token("h") {
        tensions.n03 = Some(3);
        tensions.n05 = Some(6);
    } else if stream.get_token("sus") {
        // XXX
        let pos = stream.pos;
        if let None = parse_tension(stream) {
            tensions.n11 = Some(5);
        }
        stream.pos = pos;
        tensions.n03 = None;
    } else if stream.get_token("add") {
        let Some(t) = parse_tension(stream) else {
            stream.pos = pos;
            return false;
        };
        add_tension(tensions, t);
    } else if stream.get_token("omit") || stream.get_token("no") {
        let Some(t) = parse_tension(stream) else {
            stream.pos = pos;
            return false;
        };
        omit_tension_explicit(tensions, t);
    } else if stream.get_token("alt") {
        // XXX
        tensions.n05 = None;
        tensions.n09n = None;
        tensions.n13 = Some(8);
    } else {
        return false;
    }
    true
}

fn omit_tension_explicit(tensions: &mut Tensions, t: (isize, isize)) {
    match t {
        (13, _) | (6, _) => tensions.n13 = None,
        (11, _) | (4, _) => tensions.n11 = None,
        (9, -1) | (2, -1) => tensions.n09f = None,
        (9, 0) | (2, 0) => {
            tensions.n09f = None;
            tensions.n09n = None;
            tensions.n09s = None;
        }
        (9, 1) | (2, 1) => tensions.n09s = None,
        (7, _) => tensions.n07 = None,
        (5, _) => tensions.n05 = None,
        (3, _) => tensions.n03 = None,
        _ => (),
    }
}

fn omit_tension_implicit(tensions: &mut Tensions, t: (isize, isize)) {
    match t {
        (13, _) => tensions.n05 = None,
        (11, _) => tensions.n03 = None,
        _ => (),
    }
}

fn add_tension(tensions: &mut Tensions, t: (isize, isize)) {
    match t {
        (13, s) | (6, s) => tensions.n13 = Some(9 + s),
        (11, s) | (4, s) => tensions.n11 = Some(5 + s),
        (9, -1) | (2, -1) => {
            tensions.n09n = None;
            tensions.n09f = Some(1);
        }
        (9, 0) | (2, 0) => {
            tensions.n09f = None;
            tensions.n09n = Some(2);
            tensions.n09s = None;
        }
        (9, 1) | (2, 1) => {
            tensions.n09n = None;
            tensions.n09s = Some(3);
        }
        (7, 0) => tensions.n07 = Some(tensions.n07_candidate),
        (7, s) => tensions.n07 = Some(10 + s),
        (5, s) => tensions.n05 = Some(7 + s),
        (3, s) => tensions.n03 = Some(4 + s),
        _ => (),
    }
}

fn set_tension_first(tensions: &mut Tensions, t: (isize, isize)) {
    match t {
        (5, 0) => tensions.n03 = None,
        (4, _) => tensions.n03 = None,
        (3, 0) => tensions.n05 = None,
        (2, _) => tensions.n03 = None,
        _ => (),
    }
    if t != (5, 0) && t != (3, 0) {
        add_tension(tensions, t);
        omit_tension_implicit(tensions, t);
    }
    for i in [7, 9, 11] {
        if i >= t.0 {
            break;
        }
        add_tension(tensions, (i, 0));
        omit_tension_implicit(tensions, (i, 0));
    }
}

fn parse_elements(stream: &mut Stream) -> Tensions {
    let mut tensions = Tensions::new();

    // "C-9" == "Cm9" != "C(-9)", "C+9" == "Caug9" != "C(+9)".
    parse_symbol(stream, &mut tensions);

    let mut is_first = true;
    let mut nest_level = 0;
    loop {
        if let Some(t) = parse_tension(stream) {
            if mem::replace(&mut is_first, false) {
                set_tension_first(&mut tensions, t);
            } else {
                add_tension(&mut tensions, t);
                omit_tension_implicit(&mut tensions, t);
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
    t.notes_rev(&mut notes, root);
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
            t.notes_rev(&mut notes, root);
        }
        notes.push(root);
    }

    notes.reverse();
    (stream.pos, notes)
}
