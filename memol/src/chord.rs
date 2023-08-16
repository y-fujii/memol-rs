use std::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    First,
    Tension,
    Addition,
}

struct Parser<'a> {
    text: &'a str,
    pos: usize,
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

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Parser {
            text: text,
            pos: 0,
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

    fn parse_sign(&mut self) -> isize {
        if self.get_token("-") || self.get_token("b") {
            -1
        } else if self.get_token("+") || self.get_token("#") {
            1
        } else {
            0
        }
    }

    fn parse_symbol(&mut self) -> bool {
        if self.get_token("maj") || self.get_token("Maj") || self.get_token("M") || self.get_token("^") {
            self.n07_candidate = 11;
        } else if self.get_token("m") {
            self.n03 = Some(3);
        } else if self.get_token("dim") || self.get_token("0") {
            self.n03 = Some(3);
            self.n05 = Some(6);
            self.n07_candidate = 9;
        } else if self.get_token("aug") {
            self.n05 = Some(8);
        } else if self.get_token("h") {
            self.n03 = Some(3);
            self.n05 = Some(6);
        } else {
            return false;
        }
        true
    }

    fn parse_tension(&mut self, ctx: Context) -> bool {
        let pos = self.pos;
        let sign = self.parse_sign();
        if self.get_token("13") {
            self.n13 = Some(9 + sign);
            if ctx != Context::Addition {
                self.n05 = None;
            }
            if ctx == Context::First {
                self.n05 = None;
                self.n07 = Some(self.n07_candidate);
                self.n09 = Some(2);
                self.n11 = Some(5);
            }
        } else if self.get_token("11") {
            self.n11 = Some(5 + sign);
            if ctx != Context::Addition {
                match sign {
                    -1 | 0 => self.n03 = None,
                    1 => self.n05 = None,
                    _ => unreachable!(),
                }
            }
            if ctx == Context::First {
                self.n07 = Some(self.n07_candidate);
                self.n09 = Some(2);
            }
        } else if self.get_token("9") {
            self.n09 = None;
            match sign {
                -1 => self.n09f = Some(1),
                0 => self.n09 = Some(2),
                1 => self.n09s = Some(3),
                _ => unreachable!(),
            }
            if ctx == Context::First {
                self.n07 = Some(self.n07_candidate);
            }
        } else if self.get_token("7") {
            // XXX: sign?
            self.n07 = Some(self.n07_candidate + sign);
        } else if self.get_token("6") {
            self.n06 = Some(9 + sign);
        } else if self.get_token("5") {
            self.n05 = Some(7 + sign);
            // XXX: dim5, aug5.
            if sign == 0 && ctx == Context::First {
                self.n03 = None;
            }
        } else if self.get_token("4") {
            self.n04 = Some(5 + sign);
            if ctx == Context::First {
                self.n03 = None;
            }
        } else if self.get_token("3") {
            self.n03 = Some(4 + sign);
            if ctx == Context::First {
                self.n05 = None;
            }
        } else if self.get_token("2") {
            self.n02 = Some(2 + sign);
            if ctx == Context::First {
                self.n03 = None;
            }
        } else {
            self.pos = pos;
            return false;
        }
        true
    }

    fn parse_sus_add(&mut self) {
        if self.get_token("sus2") {
            self.n03 = None;
            self.n02 = Some(2);
        } else if self.get_token("sus4") || self.get_token("sus") {
            self.n03 = None;
            self.n04 = Some(5);
        } else if self.get_token("add") {
            self.parse_tension(Context::Addition);
        }
    }

    fn parse_note(&mut self) -> Option<isize> {
        let note = if self.get_token("C") {
            0
        } else if self.get_token("D") {
            2
        } else if self.get_token("E") {
            4
        } else if self.get_token("F") {
            5
        } else if self.get_token("G") {
            7
        } else if self.get_token("A") {
            9
        } else if self.get_token("B") {
            11
        } else {
            return None;
        };
        let sign = if self.get_token("b") {
            -1
        } else if self.get_token("#") {
            1
        } else {
            0
        };
        Some(note + sign)
    }

    fn parse(&mut self) -> bool {
        self.root = self.parse_note();
        if let None = self.root {
            return false;
        }

        // "C-9" => ["C-", "9"].
        if self.get_token("-") {
            self.n03 = Some(3);
        } else if self.get_token("+") {
            self.n05 = Some(8);
        };

        let mut ctx = Context::First;
        while self.pos < self.text.len() {
            self.parse_symbol();
            if self.parse_tension(ctx) {
                ctx = Context::Tension;
            }
            self.parse_sus_add();

            if self.get_token("(") {
                ctx = Context::Tension;
            }
            // XXX: infinite loop.
        }

        // ToDo: strict syntax errors.  no, omit, dim5, aug5, on-chords, fractional chords.
        true
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

pub fn parse(text: &str) -> Vec<isize> {
    let mut parser = Parser::new(text);
    parser.parse();
    parser.notes()
}
