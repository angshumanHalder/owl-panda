// HTML parser supports only the following syntax
// 1. Balanced tags (<p></p>)
// 2. Attributes quoted  values: id="root"
// 3. Text nodes: <em>Hello</em>

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    // Read the next character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // Do the next character starts with the given string?
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // If the exact string `s` is found at the current position cosume it.
    fn expect(&mut self, s: &str) -> Result<(), String> {
        if self.starts_with(s) {
            self.pos += s.len();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?} at byte {} but it was not found",
                s, self.pos
            ))
        }
    }
}
