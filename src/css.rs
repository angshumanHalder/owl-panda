// CSS parser supports only the following syntax based on CSS2.1
// 1. Tag name
// 2. Id with prefixed by #
// 3. Any number of class names prefixed by .
// 4. Some combination of the above 3
//

use phf::phf_map;

pub const INHERITED_PROPERTY: phf::Map<&'static str, bool> = phf_map! {
    "azimuth" => true,
    "border-collapse" => true,
    "border-spacing" => true,
    "caption-side" => true,
    "color" => true,
    "cursor" => true,
    "direction" => true,
    "elevation" => true,
    "empty-cells" => true,
    "font-family" => true,
    "font-size" => true,
    "font-style" => true,
    "font-variant" => true,
    "font-weight" => true,
    "font" => true,
    "letter-spacing" => true,
    "line-height" => true,
    "list-style-image" => true,
    "list-style-position" => true,
    "list-style-type" => true,
    "list-style" => true,
    "orphans" => true,
    "pitch-range" => true,
    "pitch" => true,
    "quotes" => true,
    "richness" => true,
    "speak-header" => true,
    "speak-numeral" => true,
    "speak-punctuation" => true,
    "speak" => true,
    "speech-rate" => true,
    "stress" => true,
    "text-align" => true,
    "text-indent" => true,
    "text-transform" => true,
    "visibility" => true,
    "voice-family" => true,
    "volume" => true,
    "white-space" => true,
    "widows" => true,
    "word-spacing" => true,
};

#[derive(Debug)]
pub struct StylesSheet {
    pub rules: Vec<Rule>,
    pub origin: CSSOrigin,
}

#[derive(Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
    pub origin: CSSOrigin,
}

#[derive(Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSSOrigin {
    Author,
    User,
}

// Cannot be enum because it can be a combination of all the 3 fields on a html tag
#[derive(Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

// Key value pair separated by :
#[derive(Debug, Clone)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
    pub origin: CSSOrigin,
    pub is_important: bool,
}

// Supports only a subset of css value types. Later add more value types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Color(ColorRGBA),
    Keyword(String),
    Length(f32, Unit),
}

// Add more units
#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Px,
    Em,
    Rem,
}

// Supports rgba only for now.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ColorRGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

struct Parser {
    pos: usize,
    input: String,
}

pub type Specificity = (usize, usize, usize);

fn valid_identifier_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}

impl Parser {
    // Read the next character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // consume the character in current position & advance position
    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        c
    }

    // consume character until 'test' return results
    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    // Parse a property name or keyword
    fn parse_identfier(&mut self) -> String {
        self.consume_while(valid_identifier_char)
    }

    // return true if all input is consumed
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    // consume & discard zero / more whitespace chars
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // If the next character matches the consume it.
    fn expect_char(&mut self, c: char) -> Result<(), String> {
        if self.consume_char() != c {
            return Err(format!(
                "fn expect_char: Expected {:?} at byte {} but it was not found",
                c, self.pos
            ));
        }
        Ok(())
    }

    // Do the next character starts with the given string?
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // If the exact string `s` is found at the current position consume it.
    fn expect(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    // Parse a single simple selector, eg: `type#id.class1.class2.class3`
    // Some malformed input like ### or *foo* will parse successfully and produce weird results
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        while !self.eof() {
            match self.next_char() {
                '#' => {
                    self.consume_char();
                    selector.id = Some(self.parse_identfier());
                }
                '.' => {
                    self.consume_char();
                    selector.class.push(self.parse_identfier());
                }
                '*' => {
                    // universal selector
                    self.consume_char();
                }
                c if valid_identifier_char(c) => {
                    selector.tag_name = Some(self.parse_identfier());
                }
                _ => break,
            }
        }
        selector
    }

    fn parse_rules(&mut self, origin: CSSOrigin) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() {
                break;
            }
            if let Some(rule) = self.parse_rule(origin) {
                rules.push(rule);
            }
        }
        rules
    }

    // Parse a rule set: `<selectors> { declarations }`
    fn parse_rule(&mut self, origin: CSSOrigin) -> Option<Rule> {
        if let (Some(s), Some(d)) = (self.parse_selectors(), self.parse_declarations(origin)) {
            Some(Rule {
                selectors: s,
                declarations: d,
                origin,
            })
        } else {
            None
        }
    }

    // Parse comma separated selectors
    fn parse_selectors(&mut self) -> Option<Vec<Selector>> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(self.parse_simple_selector()));
            self.consume_whitespace();
            match self.next_char() {
                ',' => {
                    self.consume_char();
                    self.consume_whitespace();
                }
                '{' => break,
                c => {
                    println!("Unexpected character {} in selector list", c);
                    return None;
                }
            }
        }

        // Return selectors with highest specificity first, for use in matching.
        selectors.sort_by_key(|k| k.specificity());
        Some(selectors)
    }

    // Parse declarations enclosed in {...}
    fn parse_declarations(&mut self, origin: CSSOrigin) -> Option<Vec<Declaration>> {
        if let Err(e) = self.expect_char('{') {
            println!(
                "fn parse_declaractions: Expected '{{' at position {}: \n Error: {}",
                self.pos, e
            );
            return None;
        }
        let mut declarations = Vec::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '}' {
                self.consume_char();
                break;
            }
            if let Some(d) = self.parse_declaraction(origin) {
                declarations.push(d);
            } else {
                // If parsing failed skip to the next semicolor or brace
                self.consume_while(|c| c != ';' && c != '}');
                if !self.eof() && self.next_char() == ';' {
                    self.consume_char();
                }
            }
        }
        Some(declarations)
    }

    // Parse a single declaration '<property>: value'.
    fn parse_declaraction(&mut self, origin: CSSOrigin) -> Option<Declaration> {
        let name = self.parse_identfier();
        self.consume_whitespace();

        if let Err(e) = self.expect_char(':') {
            println!("fn parse_declaraction: Error: {}", e);
            return None;
        };

        self.consume_whitespace();

        let value = self.parse_value();

        if let Some(value) = value {
            let mut dec = Declaration {
                name,
                origin,
                value,
                is_important: false,
            };

            self.consume_whitespace();

            // check if important
            if self.expect("!important") {
                dec.is_important = true;
            }

            self.consume_whitespace();

            if let Err(e) = self.expect_char(';') {
                println!("{}", e);
                return None;
            };
            return Some(dec);
        }
        None
    }

    // methods to parse a value
    fn parse_value(&mut self) -> Option<Value> {
        match self.next_char() {
            '0'..='9' => self.parse_length(),
            '#' => self.parse_color(),
            _ => Some(Value::Keyword(self.parse_identfier())),
        }
    }

    fn parse_length(&mut self) -> Option<Value> {
        let f = self.parse_float();
        self.parse_unit().map(|u| Value::Length(f, u))
    }

    fn parse_float(&mut self) -> f32 {
        self.consume_while(|a| matches!(a, '0'..='9' | '.'))
            .parse()
            .unwrap()
    }

    fn parse_unit(&mut self) -> Option<Unit> {
        match &*self.parse_identfier().to_ascii_lowercase() {
            "px" => Some(Unit::Px),
            "em" => Some(Unit::Em),
            "rem" => Some(Unit::Rem),
            _ => None,
        }
    }

    fn parse_color(&mut self) -> Option<Value> {
        if let Err(e) = self.expect_char('#') {
            println!("fn parse_color: Invalid color input: {}", e);
            self.consume_while(|c| c == ';');
            return None;
        };
        Some(Value::Color(ColorRGBA {
            r: self.parse_hex_pair(),
            g: self.parse_hex_pair(),
            b: self.parse_hex_pair(),
            a: 255,
        }))
    }

    fn parse_hex_pair(&mut self) -> u8 {
        let s = &self.input[self.pos..self.pos + 2];
        self.pos += 2;
        u8::from_str_radix(s, 16).unwrap()
    }
}

impl Selector {
    pub fn specificity(&self) -> Specificity {
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}

impl Value {
    pub fn to_px(&self) -> f32 {
        match *self {
            Value::Length(f, Unit::Px) => f,
            // TODO: to convert other units implement inheritance
            _ => 0.0,
        }
    }
}

pub fn parse(source: String, origin: CSSOrigin) -> StylesSheet {
    let mut parser = Parser {
        pos: 0,
        input: source,
    };
    StylesSheet {
        rules: parser.parse_rules(origin),
        origin,
    }
}
