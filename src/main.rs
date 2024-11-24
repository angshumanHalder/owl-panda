use std::fs;

mod css;
mod dom;
mod html;
mod layout;
mod painting;
mod style;

fn main() {
    let html = read_source(String::from("files/test.html"));
    let root_node = html::parse(html);

    let author_css = read_source(String::from("files/test.css"));
    let user_css = read_source(String::from("files/default.css"));
    let author_rules = css::parse(author_css, css::CSSOrigin::Author);
    let user_rules = css::parse(user_css, css::CSSOrigin::User);
    println!("{:#?}", user_rules);

    let stylesheets = vec![author_rules, user_rules];
    let style_tree = style::style_tree(&root_node, &stylesheets, None);
    println!("{:#?}", style_tree);
}

fn read_source(file_path: String) -> String {
    fs::read_to_string(file_path).expect("File should exists")
}
