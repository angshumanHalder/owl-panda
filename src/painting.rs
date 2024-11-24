// The rastarizer only paints one thing: rectangles.
// This doesn't support z-index

use crate::{
    css::ColorRGBA,
    css::Value,
    layout::{BoxType, LayoutBox, Rect},
};

type DisplayList = Vec<DisplayCommand>;

enum DisplayCommand {
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
            width: border_box.width,
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
            height: border_box.height,
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
