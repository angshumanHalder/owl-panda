use std::fs;

mod css;
mod dom;
mod html;

fn main() {
    let html = read_source(String::from("files/test.html"));
    let root_node = html::parse(html);
    println!("{:#?}", root_node);

    let css = read_source(String::from("files/test.css"));
    let rules = css::parse(css);
    println!("{:#?}", rules);
}

fn read_source(file_path: String) -> String {
    fs::read_to_string(file_path).expect("File should exists")
}
