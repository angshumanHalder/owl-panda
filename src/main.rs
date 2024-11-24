use std::{
    fs::{self, File},
    io::BufWriter,
};

mod css;
mod dom;
mod html;
mod layout;
mod painting;
mod style;

fn main() {
    let mut opts = getopts::Options::new();
    opts.optopt("h", "html", "HTML Document", "[FILENAME]");
    opts.optopt("c", "css", "CSS Document", "[FILENAME]");
    opts.optopt("o", "output", "Output file", "[FILENAME]");

    let matches = opts.parse(std::env::args().skip(1)).unwrap();
    let str_args = |flag: &str, default: &str| -> String {
        matches.opt_str(flag).unwrap_or(default.to_string())
    };

    // Read input files
    let html = read_source(str_args("h", "files/test.html"));
    let author_css = read_source(str_args("c", "files/test.css"));
    let user_css = read_source(String::from("files/default.css"));

    // setup a viewport due to lack of actual window
    let mut viewport: layout::Dimensions = Default::default();
    viewport.content.width = 800.0;
    viewport.content.height = 600.0;

    // Parsing & rendering
    let root_node = html::parse(html);
    let author_rules = css::parse(author_css, css::CSSOrigin::Author);
    let user_rules = css::parse(user_css, css::CSSOrigin::User);
    let stylesheets = vec![author_rules, user_rules];
    let style_root = style::style_tree(&root_node, &stylesheets, None);
    let layout_root = layout::layout_tree(&style_root, viewport);

    // create output file
    let filename = str_args("o", "output.png");
    let mut file = BufWriter::new(File::create(&filename).unwrap());

    let canvas = painting::paint(&layout_root, viewport.content);
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let img = image::ImageBuffer::from_fn(w, h, move |x, y| {
        let color = canvas.pixels[(y * w + x) as usize];
        image::Rgba([color.r, color.g, color.b, color.a])
    });
    let ok = image::DynamicImage::ImageRgba8(img)
        .write_to(&mut file, image::ImageFormat::Png)
        .is_ok();

    if ok {
        println!("Saved output as {}", filename)
    } else {
        println!("Error saving output as {}", filename)
    }
}

fn read_source(file_path: String) -> String {
    fs::read_to_string(file_path).expect("File should exists")
}
