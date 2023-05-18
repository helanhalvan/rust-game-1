use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

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

mod icons;
mod util;

type Myrgb = palette::rgb::Rgb<Srgb, f64>;

const BASE: &str = "./img/";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ColorSource {
    a: f32,
    b: f32,
    c: f32,
}

#[derive(Debug, Clone)]
pub(super) struct ImageSourceData {
    pub name: String,
    pub data: HashMap<String, i32>,
}

fn get_color_pair(c: CellState) -> (Myrgb, Myrgb) {
    let dir = "".to_string() + BASE + &c.variant.to_string();
    util::get_with_file_cache(&dir, "/colors.json", color_pair, new_color_pair)
}

fn get_glyth_map() -> HashMap<String, i32> {
    let dir = "".to_string() + BASE;
    util::get_with_file_cache(&dir, "/glyth_map.json", |i| i, HashMap::new)
}

fn new_color_pair() -> ColorSource {
    let mut rng = rand::thread_rng();
    ColorSource {
        a: rng.gen(),
        b: rng.gen(),
        c: rng.gen(),
    }
}

fn cellstate_to_img_src_data(c: CellState) -> ImageSourceData {
    let name = c.variant.to_string();
    let data = match c.data {
        CellStateData::Unit => HashMap::from([("unit".to_string(), 1)]),
        CellStateData::Slot {
            slot: celldata::Slot::Done,
        } => HashMap::from([("done".to_string(), 1)]),
        CellStateData::Slot {
            slot: celldata::Slot::Empty,
        } => HashMap::from([("empty".to_string(), 1)]),
        CellStateData::InProgress(actionmachine::InProgress::Pure(countdown))
        | CellStateData::InProgress(actionmachine::InProgress::WithOther(countdown, _)) => {
            HashMap::from([("InProgress".to_string(), countdown as i32)])
        }
        CellStateData::Resource(Resource::Pure(resources))
        | CellStateData::Resource(Resource::WithVariant(resources, _)) => {
            let map = resource::to_key_value_display_amounts(c.variant, resources);
            map.into_iter()
                .map(|(k, v)| (format!("{:?}", k), v))
                .collect()
        }
    };
    ImageSourceData { name, data }
}

pub(crate) fn make_image(c: CellState) -> PathBuf {
    let sd = cellstate_to_img_src_data(c);
    let path = make_path(sd.clone());
    if let Ok(_) = fs::read(&path) {
        print!("x");
        return path;
    }
    let (background_color, front_color) = get_color_pair(c);
    let glyth_dir = "".to_string() + BASE + &sd.name + "/glyths/";
    icons::setup_alphabets(&glyth_dir, background_color, front_color);

    let mut glyth_ids = get_glyth_map();
    let mut next_id = if let Some(max) = glyth_ids.values().max() {
        *max + 1
    } else {
        0
    };
    let glyth_map = sd
        .data
        .iter()
        .map(|(k, _)| {
            if let Some(v) = glyth_ids.get(k) {
                (k.clone(), *v)
            } else {
                let ret = (k.clone(), next_id);
                glyth_ids.insert(k.clone(), next_id);
                next_id = next_id + 1;
                ret
            }
        })
        .map(|(name, id)| {
            if let Some(path) = icons::get_synt_glyth_path(id, &glyth_dir) {
                (name, path)
            } else {
                (name, icons::get_real_glyth_path(id, &glyth_dir))
            }
        })
        .collect();
    //let icon = get_id(glyth_map, icon_string);
    //let img_path = icons::get_synth_glyth(icon, glyth_dir);

    make_image_int(path.clone(), sd, glyth_map, (background_color, front_color));
    path
}

fn make_image_int(
    path: PathBuf,
    sd: ImageSourceData,
    glyth_map: HashMap<String, PathBuf>,
    (background_color, front_color): (Myrgb, Myrgb),
) {
    let name = sd.name;
    let width = (visualize_cell::START_CELL_Y_SIZE * 4.0) as i32;
    let height = (visualize_cell::START_CELL_X_SIZE * 4.0) as i32;
    let fontsize = height as f64 / 8.0;
    let spacing = height as f64 / 40.0;
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

    let number_of_resources = sd.data.keys().count() as f64;
    let icon_radius = (spacing * 6.0) / number_of_resources;
    let columns = number_of_resources as i32 * 2;
    context = sd
        .data
        .into_iter()
        .fold(context, |mut context, (s, value)| {
            context = draw_x(context, value, icon_radius, s, columns, &glyth_map);
            context.translate(width as f64 / number_of_resources, 0.0);
            context
        });

    let mut file = File::create(&path).expect("Couldn't create 'file.png'");
    match surface.write_to_png(&mut file) {
        Ok(_) => print!("."),
        Err(_) => println!("Error create file.png"),
    }
}

fn draw_x(
    mut context: Context,
    x: i32,
    radius: f64,
    icon: String,
    columns: i32,
    glyth_map: &HashMap<String, PathBuf>,
) -> Context {
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
        context = draw_icon(context, radius, icon.clone(), glyth_map);
        context.translate(size, 0.0);
    }
    let _ = context.restore();
    context
}

fn draw_icon(
    context: Context,
    radius: f64,
    icon_string: String,
    glyth_map: &HashMap<String, PathBuf>,
) -> Context {
    let img_path = glyth_map.get(&icon_string).unwrap();
    let mut src = fs::File::open(img_path).unwrap();
    let icon_surface = cairo::ImageSurface::create_from_png(&mut src).unwrap();
    let _ = context.save();
    context.translate(-radius, -radius);
    context.scale((radius * 2.0) / 600.0, (radius * 2.0) / 600.0);
    let _ = context.set_source_surface(&icon_surface, 0.0, 0.0);
    let _ = context.paint();
    let _ = context.restore();

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

fn make_path(sd: ImageSourceData) -> PathBuf {
    let base = "".to_string() + BASE + &sd.name + "/img/";
    let _ = fs::create_dir_all(base.clone());
    let name = sd
        .data
        .into_iter()
        .sorted()
        .map(|(t, v)| t + ":" + &v.to_string())
        .join("_");
    let path = base + &name + &".png";
    PathBuf::from(path)
}
