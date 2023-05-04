use std::fs;
use std::fs::File;

use cairo::Format;

use cairo::Context;

use itertools::Itertools;
use palette::{FromColor, Hsl, Srgb};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::celldata::{CellState, CellStateData, CellStateVariant};
use crate::resource::Resource;
use crate::{actionmachine, visualize_cell};
use crate::{
    celldata::{self},
    resource,
};

type Myrgb = palette::rgb::Rgb<Srgb, f64>;

const BASE: &str = "./img/";

#[derive(Serialize, Deserialize, Debug)]
struct ColorSource {
    a: f32,
    b: f32,
    c: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Icon {
    BrokenCircle,
    Triangle,
    OtherTriangle,
}

fn get_color_pair(cv: CellStateVariant) -> (Myrgb, Myrgb) {
    let dir = "".to_string() + BASE + &cv.to_string();
    let path = dir.clone() + "/colors.json";
    if let Ok(f) = fs::read_to_string(&path) {
        let p: ColorSource = serde_json::from_str(&f).unwrap();
        color_pair(p)
    } else {
        let mut rng = rand::thread_rng();
        let p = ColorSource {
            a: rng.gen(),
            b: rng.gen(),
            c: rng.gen(),
        };
        let cont = serde_json::to_string(&p).unwrap();
        let _ = fs::create_dir_all(dir);
        fs::write(path, cont).unwrap();
        color_pair(p)
    }
}

pub(crate) fn make_image(c: CellState) -> String {
    let path = make_path(c);
    if let Ok(_) = fs::read(&path) {
        print!("x");
        return path;
    }
    let (background_color, front_color) = get_color_pair(c.variant);
    make_image_int(c, (background_color, front_color));
    path
}

fn make_image_int(c: CellState, (background_color, front_color): (Myrgb, Myrgb)) {
    let name = c.variant.to_string();
    let width = (visualize_cell::START_CELL_Y_SIZE * 4.0) as i32;
    let height = (visualize_cell::START_CELL_X_SIZE * 4.0) as i32;
    let fontsize = height as f64 / 8.0;
    let spacing = height as f64 / 40.0;
    let path = make_path(c);
    let surface = cairo::ImageSurface::create(Format::Rgb24.into(), width, height).unwrap();
    let mut context = cairo::Context::new(&surface).unwrap();
    context = set_color(context, background_color);
    context.rectangle(0., 0., f64::from(width), f64::from(height));
    let _ = context.fill();
    context.set_line_width(spacing);
    context = set_color(context, front_color);

    context.select_font_face(
        "Monaco", // TODO find cool font
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );
    context.set_font_size(fontsize as f64);
    let top_y = spacing * 3.0;
    //context.text_extents(&name); // dont know what this does or why it would be needed
    context.move_to(spacing as f64, (spacing + fontsize) as f64);
    let _ = context.show_text(&name);
    context.translate(0.0, fontsize);
    context.move_to(0.0, 0.0);
    let _ = context.fill();

    match c.data {
        CellStateData::Unit => {
            context.translate(width as f64 / 2.0, height as f64 / 2.0);
            draw_icon(context, spacing * 8.0, Icon::OtherTriangle);
        }
        CellStateData::InProgress(actionmachine::InProgress::Pure(countdown))
        | CellStateData::InProgress(actionmachine::InProgress::WithOther(countdown, _)) => {
            draw_x(
                context,
                countdown as i32,
                spacing * 4.0,
                Icon::BrokenCircle,
                6,
            );
        }
        CellStateData::Slot {
            slot: celldata::Slot::Done,
        } => {
            context.move_to(spacing, top_y);
            context.line_to(width as f64 / 2.0, height as f64 - spacing - fontsize);
            context.line_to(width as f64 - spacing, top_y);
            context.line_to(spacing, top_y);
            context.close_path();
            let _ = context.fill();
        }
        CellStateData::Slot {
            slot: celldata::Slot::Empty,
        } => {
            context.move_to(spacing, top_y);
            context.line_to(width as f64 / 2.0, height as f64 - spacing - fontsize);
            context.line_to(width as f64 - spacing, top_y);
            context.line_to(spacing, top_y);
            context.close_path();
            let _ = context.stroke();
        }
        CellStateData::Resource(Resource::Pure(resources))
        | CellStateData::Resource(Resource::WithVariant(resources, _)) => {
            let map = resource::to_key_value_display_amounts(c.variant, resources);
            let number_of_resources = map.keys().count() as f64;
            let icon_radius = (spacing * 6.0) / number_of_resources;
            let columns = number_of_resources as i32 * 2;
            match map.get(&resource::ResourceType::Builders) {
                Some(value) => {
                    context = draw_x(context, *value, icon_radius, Icon::BrokenCircle, columns);
                    context.translate(width as f64 / number_of_resources, 0.0)
                }
                None => {}
            };
            match map.get(&resource::ResourceType::LogisticsPoints) {
                Some(value) => {
                    context = draw_x(context, *value, icon_radius, Icon::Triangle, columns);
                    context.translate(width as f64 / number_of_resources, 0.0)
                }
                None => {}
            }

            match map.get(&resource::ResourceType::BuildTime) {
                Some(value) => {
                    context = draw_x(context, *value, icon_radius, Icon::Triangle, columns);
                    context.translate(width as f64 / number_of_resources, 0.0)
                }
                None => {}
            }

            match map.get(&resource::ResourceType::IronOre) {
                Some(value) => {
                    context = draw_x(context, *value, icon_radius, Icon::Triangle, columns);
                    context.translate(width as f64 / number_of_resources, 0.0)
                }
                None => {}
            }

            match map.get(&resource::ResourceType::Wood) {
                Some(value) => {
                    draw_x(context, *value, icon_radius, Icon::OtherTriangle, columns);
                }
                None => {}
            }
        }
    };

    let mut file = File::create(&path).expect("Couldn't create 'file.png'");
    match surface.write_to_png(&mut file) {
        Ok(_) => print!("."),
        Err(_) => println!("Error create file.png"),
    }
}

fn draw_x(mut context: Context, x: i32, radius: f64, icon: Icon, columns: i32) -> Context {
    let row_length = 3;
    let _ = context.save();
    let size = radius * 2.5;
    context.translate(size / 2.0, size * 0.75);
    if x > (row_length * columns) {
        let _ = context.show_text(&x.to_string());
        let _ = context.fill();
        return context;
    }
    for j in 0..x {
        if (j % row_length == 0) && (j != 0) {
            context.translate(row_length as f64 * -size, size);
        }
        context = draw_icon(context, radius, icon);
        context.translate(size, 0.0);
    }
    let _ = context.restore();
    context
}

fn draw_icon(context: Context, radius: f64, icon: Icon) -> Context {
    // TODO sane icon names and nice looking icons
    match icon {
        Icon::BrokenCircle => {
            context.arc(0.0, 0.0, radius, 30.0, 1.0 * std::f64::consts::PI);
        }
        Icon::Triangle => {
            context.arc(0.0, 0.0, radius, 0.0, 2.0 * std::f64::consts::PI);
        }
        Icon::OtherTriangle => {
            context.arc(0.0, 0.0, radius, 0.0, 1.0 * std::f64::consts::PI);
        }
    };
    let _ = context.fill();
    context
}

fn set_color(context: Context, color: Myrgb) -> Context {
    context.set_source_rgb(color.red, color.green, color.blue);
    context
}

fn color_pair(ColorSource { a, b, c }: ColorSource) -> (Myrgb, Myrgb) {
    let y: f32 = a;
    let hue: f32 = y * 360.0;
    let saturation: f32 = f32::max(b, 0.6);
    let value: f32 = f32::max(c, 0.6);
    let hsv: palette::Hsv<f64> = palette::Hsv::new(hue, saturation, value);
    let hsl = Hsl::from_color(hsv);

    let complement = hsl.hue + 180.0 % 360.0;
    let l = hsl.lightness;

    let lighter = l + 0.30;
    let darker = l - 0.30;

    let background: Hsl<_> = Hsl::new(hsl.hue, hsl.saturation, lighter);
    let front = Hsl::new(complement, hsl.saturation, darker);
    (hsl_to_rgb(background), hsl_to_rgb(front))
}

fn hsl_to_rgb(hsl: Hsl) -> Myrgb {
    let rgb = palette::rgb::Rgb::from_color(hsl);
    palette::rgb::Rgb::from_components((rgb.red.into(), rgb.green.into(), rgb.blue.into()))
}

fn make_path(cellstate: celldata::CellState) -> String {
    let cv = cellstate.variant;
    let base = "".to_string() + BASE + &cv.to_string();
    let (dir, path) = match cellstate.data {
        CellStateData::InProgress(actionmachine::InProgress::Pure(countdown))
        | CellStateData::InProgress(actionmachine::InProgress::WithOther(countdown, _)) => {
            let dir = base + "/inprogress/";
            (dir.clone(), dir + &countdown.to_string() + &".png")
        }
        celldata::CellStateData::Resource(Resource::Pure(r))
        | celldata::CellStateData::Resource(Resource::WithVariant(r, _)) => {
            let name = resource::to_key_value_display_amounts(cv, r)
                .into_iter()
                .sorted()
                .map(|(t, v)| format!("{:?}", t) + ":" + &v.to_string())
                .join("_");
            let dir = base + "/resource/";
            (dir.clone(), dir + "a_" + &name + &".png")
        }
        celldata::CellStateData::Unit { .. } => {
            let dir = base + "/unit/";
            (dir.clone(), dir + &"unit.png")
        }
        celldata::CellStateData::Slot {
            slot: celldata::Slot::Empty,
            ..
        } => {
            let dir = base + "/empty/";
            (dir.clone(), dir + &"empty.png")
        }
        celldata::CellStateData::Slot {
            slot: celldata::Slot::Done,
            ..
        } => {
            let dir = base + "/done/";
            (dir.clone(), dir + "done" + &".png")
        }
    };
    let _ = fs::create_dir_all(dir.clone());
    path
}
