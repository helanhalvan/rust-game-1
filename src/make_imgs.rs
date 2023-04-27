use std::fs::File;
use std::{collections::HashMap, fs};

use cairo::Format;

use cairo::{Context};

use palette::{FromColor, Hsl, Srgb};

use rand::Rng;


use crate::celldata::{CellState, CellStateData};
use crate::visualize_cell;
use crate::{
    celldata::{self},
    resource,
};

type Myrgb = palette::rgb::Rgb<Srgb, f64>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Icon {
    BrokenCircle,
    Triangle,
    OtherTriangle,
}

/*
CellState {
    variant: Hub,
    data: Resource {
        resources: [
            ResourceData {
                current: 9,
                max: 9,
            },
            ResourceData {
                current: 0,
                max: 0,
            },
            ResourceData {
                current: 2,
                max: 3,
            },
        ],
    },
}
 */

pub fn all_imgs() -> Vec<(celldata::CellState, String)> {
    let mut all_imgs_buff = Vec::new();
    for c in celldata::non_interactive_statespace() {
        all_imgs_buff.push((c, make_path(c)))
    }
    return all_imgs_buff;
}

pub fn make_imgs() {
    for (variant, vec) in statespace_groups() {
        let (background_color, front_color) = random_color_pair();
        let name = variant.to_string();
        let width = (visualize_cell::CELL_Y_SIZE * 4.0) as i32;
        let height = (visualize_cell::CELL_X_SIZE * 4.0) as i32;
        let fontsize = height as f64 / 8.0;
        let spacing = height as f64 / 40.0;

        for i in vec {
            let path = make_path(i);
            let surface = cairo::ImageSurface::create(Format::Rgb24.into(), width, height).unwrap();
            let mut context = cairo::Context::new(&surface).unwrap();
            context = set_color(context, background_color);
            context.rectangle(0., 0., f64::from(width), f64::from(height));
            let _ = context.fill();
            context.set_line_width(1.0);
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

            match i.data {
                CellStateData::Unit => {
                    // TODO nicer (possibly randomised) icon here
                    context.move_to(spacing, top_y);
                    context.line_to(width as f64 / 2.0, height as f64 - spacing - fontsize);
                    context.line_to(width as f64 - spacing, top_y);
                    context.line_to(spacing, top_y);
                    context.close_path();
                    let _ = context.fill();
                }
                CellStateData::InProgress { countdown, .. } => {
                    context = draw_x(context, countdown as i32, spacing * 4.0, Icon::BrokenCircle);
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
                CellStateData::Resource { resources } => {
                    let map = resource::to_key_value(resources);
                    match map.get(&resource::ResouceType::Builders) {
                        Some(value) => {
                            context = draw_x(context, *value, spacing * 2.0, Icon::BrokenCircle)
                        }

                        None => {}
                    };
                    context.translate(width as f64 / 2.0, 0.0);
                    match map.get(&resource::ResouceType::LogisticsPoints) {
                        Some(value) => {
                            context = draw_x(context, *value, spacing * 2.0, Icon::Triangle)
                        }

                        None => {}
                    }
                }
            };

            let mut file = File::create(&path).expect("Couldn't create 'file.png'");
            match surface.write_to_png(&mut file) {
                Ok(_) => println!("{}, created", path),
                Err(_) => println!("Error create file.png"),
            }
        }
    }
}

fn draw_x(mut context: Context, x: i32, radius: f64, icon: Icon) -> Context {
    let _ = context.save();
    let size = radius * 2.5;
    context.translate(size, size);
    let row_length = 4;
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
    match icon {
        Icon::BrokenCircle => {
            context.arc(0.0, 0.0, radius, 30.0, 1.0 * std::f64::consts::PI);
        }
        Icon::Triangle => {
            // TODO sane icon names and nice looking icons
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

fn random_color_pair() -> (Myrgb, Myrgb) {
    let mut rng = rand::thread_rng();
    let y: f32 = rng.gen(); // generates a float between 0 and 1
    let hue: f32 = y * 360.0;
    let saturation: f32 = f32::max(rng.gen(), 0.6);
    let value: f32 = f32::max(rng.gen(), 0.6);
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

fn statespace_groups() -> HashMap<celldata::CellStateVariant, Vec<CellState>> {
    celldata::non_interactive_statespace().into_iter().fold(
        HashMap::new(),
        |mut acc: HashMap<celldata::CellStateVariant, Vec<_>>, i| {
            let key = i.variant;
            if let Some(v) = acc.get_mut(&key) {
                v.push(i);
                acc
            } else {
                let mut v = vec![];
                v.push(i);
                acc.insert(key, v);
                acc
            }
        },
    )
}

fn make_path(cellstate: celldata::CellState) -> String {
    let cv = cellstate.variant;
    let base = "./img/".to_string();
    let (dir, path) = match cellstate.data {
        celldata::CellStateData::InProgress { countdown, .. } => {
            let dir = base + &cv.to_string() + "/inprogress/";
            (dir.clone(), dir + &countdown.to_string() + &".png")
        }
        celldata::CellStateData::Resource { resources: r, .. } => {
            let b = resource::get(resource::ResouceType::Builders, r);
            let lp = resource::get(resource::ResouceType::LogisticsPoints, r);
            let dir = base + &cv.to_string() + "/resource/";
            (
                dir.clone(),
                dir + &b.to_string() + ":" + &lp.to_string() + &".png",
            )
        }
        celldata::CellStateData::Unit { .. } => {
            let dir = base + &cv.to_string() + "/unit/";
            (dir.clone(), dir + &"unit.png")
        }
        celldata::CellStateData::Slot {
            slot: celldata::Slot::Empty,
            ..
        } => {
            let dir = base + &cv.to_string() + "/empty/";
            (dir.clone(), dir + &"empty.png")
        }
        celldata::CellStateData::Slot {
            slot: celldata::Slot::Done,
            ..
        } => {
            let dir = base + &cv.to_string() + "/done/";
            (dir.clone(), dir + "done" + &".png")
        }
    };
    let _ = fs::create_dir_all(dir.clone());
    path
}
