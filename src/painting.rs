// The rastarizer only paints one thing: rectangles.
// This doesn't support z-index

use crate::{
    css::ColorRGBA,
    css::Value,
    layout::{BoxType, LayoutBox, Rect},
};

type DisplayList = Vec<DisplayCommand>;

pub enum DisplayCommand {
    SolidColor(ColorRGBA, Rect),
}

pub fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_root: &LayoutBox) {
    render_background(list, layout_root);
    render_borders(list, layout_root);

    for child in &layout_root.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    if let Some(c) = get_color(layout_box, "background") {
        list.push(DisplayCommand::SolidColor(
            c,
            layout_box.dimensions.border_box(),
        ));
    }
}

// Returns specified color for property `name` or None if no color is found
fn get_color(layout_box: &LayoutBox, name: &str) -> Option<ColorRGBA> {
    match layout_box.box_type {
        BoxType::BlockNode(style) | BoxType::InlineNode(style) => match style.value(name) {
            Some(Value::Color(c)) => Some(c),
            _ => None,
        },
        BoxType::AnonymousBlock => None,
    }
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    let color = match get_color(layout_box, "border-color") {
        Some(c) => c,
        _ => return,
    };

    let d = &layout_box.dimensions;
    let border_box = d.border_box();

    // left
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: d.border.left,
            height: border_box.height,
        },
    ));

    // right
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x + border_box.width - d.border.right,
            y: border_box.y,
            width: d.border.right,
            height: border_box.height,
        },
    ));

    // top
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: border_box.width,
            height: d.border.top,
        },
    ));

    // bottom
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y + border_box.height - d.border.bottom,
            width: border_box.width,
            height: d.border.bottom,
        },
    ));
}

pub struct Canvas {
    pub pixels: Vec<ColorRGBA>,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    /// Create a blank canvas
    fn new(width: usize, height: usize) -> Canvas {
        let white = ColorRGBA {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        };
        Canvas {
            pixels: vec![white; width * height],
            width,
            height,
        }
    }

    fn paint_item(&mut self, item: &DisplayCommand) {
        match *item {
            DisplayCommand::SolidColor(color, rect) => {
                // Clip the rectangle to the canvas boundaries.
                let x0 = rect.x.clamp(0.0, self.width as f32) as usize;
                let y0 = rect.y.clamp(0.0, self.height as f32) as usize;
                let x1 = (rect.x + rect.width).clamp(0.0, self.width as f32) as usize;
                let y1 = (rect.y + rect.height).clamp(0.0, self.height as f32) as usize;

                for y in y0..y1 {
                    for x in x0..x1 {
                        // TODO: alpha compositing with existing pixel
                        self.pixels[y * self.width + x] = color;
                    }
                }
            }
        }
    }
}

pub fn paint(layout_root: &LayoutBox, bounds: Rect) -> Canvas {
    let display_list = build_display_list(layout_root);
    let mut canvas = Canvas::new(bounds.width as usize, bounds.height as usize);
    for item in display_list {
        canvas.paint_item(&item);
    }
    canvas
}
