// CSS parser supports only the following syntax based on CSS2.1
// 1. Tag name
// 2. Id with prefixed by #
// 3. Any number of class names prefixed by .
// 4. Some combination of the above 3

struct StylesSheet {
    rules: Vec<Rule>,
}

struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

enum Selector {
    Simple(SimpleSelector),
}

// Cannot be enum because it can be a combination of all the 3 fields on a html tag
struct SimpleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    class: Vec<String>,
}

// Key value pair separated by :
struct Declaration {
    key: String,
    value: Value,
}

// Supports only a subset of css value types. Later add more value types
enum Value {
    Color(ColorRGBA),
    Keyword(String),
    Length(f32, Unit),
}

// Add more units
enum Unit {
    Px,
}

// Supports rgba only for now.
struct ColorRGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    // Parse a single simple selector, eg: `type#id.class1.class2.class3`
    // Some malformed input like ### or *foo* will parse successfully and produce weird results
    fn parse_simple_selector(&mut self) -> SimpleSelector {
        let mut selector = SimpleSelector {
            tag_name: None,
            id: None,
            class: Vec::new(),
        };
        todo!()
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}
