// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

struct Stream<'a> {
    text: &'a str,
    pos: usize,
}

struct Tensions {
    acc: [isize; 7],
    use_: [bool; 7],
    alt9: [bool; 2],
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
            acc: [0; 7],
            use_: [false; 7],
            alt9: [false; 2],
        }
    }

    fn set_acc(&mut self, (n, acc): (usize, isize)) {
        // XXX
        if n >= 7 || acc != 0 {
            self.acc[n % 7] = acc;
        }
    }

    fn set_use(&mut self, (n, acc): (usize, isize), use_: bool) {
        self.use_[n % 7] = use_;
        if n % 7 == 1 && acc != 0 {
            self.alt9[((1 + acc) / 2) as usize] = use_;
        }
    }

    fn omit_by(&mut self, (n, _): (usize, isize)) {
        if n >= 7 {
            self.use_[n - 8] = false;
        }
    }

    fn get_notes_rev(&self, dst: &mut Vec<isize>, root: isize) {
        let mixolydian = [0, 2, 4, 5, 7, 9, 10];
        for i in (0..7).rev() {
            if i == 1 && self.acc[1] != 0 {
                if self.alt9[1] {
                    dst.push(root + 3);
                }
                if self.alt9[0] {
                    dst.push(root + 1);
                }
            } else if self.use_[i] {
                dst.push(root + mixolydian[i] + self.acc[i]);
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
    if stream.get_token("madd") || stream.get_token("maug") || stream.get_token("malt") {
        stream.pos = pos;
        stream.get_token("m");
        tensions.acc[2] = -1;
    } else if stream.get_token("maj") || stream.get_token("ma") || stream.get_token("M") || stream.get_token("^") {
        tensions.acc[6] = 1;
    } else if stream.get_token("min") || stream.get_token("mi") || stream.get_token("m") || stream.get_token("-") {
        tensions.acc[2] = -1;
    } else if stream.get_token("dim") || stream.get_token("0") {
        tensions.acc[2] = -1;
        tensions.acc[4] = -1;
        tensions.acc[6] = -1;
    } else if stream.get_token("aug") || stream.get_token("+") {
        tensions.acc[2] = 0;
        tensions.acc[4] = 1;
    } else if stream.get_token("h") {
        tensions.acc[2] = -1;
        tensions.acc[4] = -1;
    } else if stream.get_token("sus") {
        // XXX
        let pos = stream.pos;
        if let None = parse_tension(stream) {
            tensions.use_[3] = true;
        }
        stream.pos = pos;
        tensions.use_[2] = false;
    } else if stream.get_token("add") {
        let Some(t) = parse_tension(stream) else {
            stream.pos = pos;
            return false;
        };
        tensions.set_acc(t);
        tensions.set_use(t, true);
    } else if stream.get_token("omit") || stream.get_token("no") {
        let Some(t) = parse_tension(stream) else {
            stream.pos = pos;
            return false;
        };
        tensions.set_use(t, false);
    } else if stream.get_token("alt") {
        // XXX
        tensions.acc = [0, 1, 0, 1, 1, -1, 0];
    } else {
        return false;
    }
    true
}

fn set_tension_first(tensions: &mut Tensions, t: (usize, isize)) {
    match t {
        (1, _) => tensions.use_[2] = false,
        (2, 0) => tensions.use_[4] = false,
        (3, _) => tensions.use_[2] = false,
        (4, 0) => tensions.use_[2] = false,
        _ => (),
    }
    for i in [6, 8, 10] {
        if i >= t.0 {
            break;
        }
        tensions.set_use((i, 0), true);
        tensions.omit_by((i, 0));
    }
}

fn parse_elements(stream: &mut Stream) -> Tensions {
    let mut tensions = Tensions::new();
    //tensions.use_[0] = true;
    tensions.use_[2] = true;
    tensions.use_[4] = true;

    // "C-9" == "Cm9" != "C(-9)", "C+9" == "Caug9" != "C(+9)".
    parse_symbol(stream, &mut tensions);

    let mut is_first = true;
    let mut nest_level = 0;
    loop {
        if let Some(t) = parse_tension(stream) {
            if mem::replace(&mut is_first, false) {
                set_tension_first(&mut tensions, t);
            }
            tensions.set_acc(t);
            tensions.set_use(t, true);
            tensions.omit_by(t);
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
