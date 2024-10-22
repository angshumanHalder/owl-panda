use std::fs;

mod dom;
mod html;

fn main() {
    let html = read_source(String::from("files/test.html"));
    let root_node = html::parse(html);
    println!("{:#?}", root_node);
}

fn read_source(file_path: String) -> String {
    fs::read_to_string(file_path).expect("File should exists")
}
