// HTML parser supports only the following syntax
// 1. Balanced tags (<p></p>)
// 2. Attributes quoted  values: id="root"
// 3. Text nodes: <em>Hello</em>
// Comments of type <!-- .. --> (the comment text cannot have '-')

use std::{char, collections::HashMap};

use crate::dom;

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

    // If the exact string `s` is found at the current position consume it.
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

    // return true if all input is consumed
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    // consume the character in current position & advance position
    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        c
    }

    // consume characters until `test` returns false
    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    // consume & discard zero / more whitespace chars
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // Parse a tag or attribute name
    fn parse_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }

    // Parse a single node
    fn parse_node(&mut self) -> Option<dom::Node> {
        if self.starts_with("<") {
            self.parse_element()
        } else {
            Some(self.parse_text())
        }
    }

    // Parse text node (simplified version)
    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }

    // Parse a single element (opening tag, contents & closing tag)
    // Parse comment only when <!-- .. -->
    fn parse_element(&mut self) -> Option<dom::Node> {
        // Opening tag
        match self.expect("<") {
            Ok(()) => (),
            Err(_) => return None,
        }

        // determine whether it is a comment or a node
        if self.starts_with("!--") {
            return self.parse_comment();
        }

        let tag_name = self.parse_name();
        let attrs = self.parse_attributes();

        match self.expect(">") {
            Ok(()) => (),
            Err(_) => return None,
        }

        // contents
        let children = self.parse_nodes();

        // Closing tag
        match self.expect(&format!("</{tag_name}>")) {
            Ok(()) => (),
            Err(_) => return None,
        }

        Some(dom::elem(tag_name, attrs, children))
    }

    // Parse Attributes - list of name="value" separated by whitespace
    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            if let Some((name, value)) = self.parse_attr() {
                attributes.insert(name, value);
            };
        }
        attributes
    }

    // Parse name="value" attribute.
    fn parse_attr(&mut self) -> Option<(String, String)> {
        let name = self.parse_name();
        match self.expect("=") {
            Ok(()) => (),
            Err(_) => return None,
        }
        let value = self.parse_attr_value();
        Some((name, value))
    }

    // Parse a "value"
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        let close_quote = self.consume_char();
        assert_eq!(open_quote, close_quote);
        value
    }

    // Parse sibling nodes
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            if let Some(node) = self.parse_node() {
                nodes.push(node);
            }
        }
        nodes
    }

    // Parse parse comment
    fn parse_comment(&mut self) -> Option<dom::Node> {
        match self.expect("!--") {
            Ok(()) => (),
            Err(_) => return None,
        }

        let text = self.consume_while(|c| c != '-');

        match self.expect("-->") {
            Ok(()) => (),
            Err(_) => return None,
        }

        Some(dom::comment(text))
    }
}

// Parse html document
pub fn parse(source: String) -> dom::Node {
    let mut nodes = Parser {
        pos: 0,
        input: source,
    }
    .parse_nodes();

    if nodes.len() == 1 {
        nodes.remove(0)
    } else {
        dom::elem(String::from("html"), HashMap::new(), nodes)
    }
}
