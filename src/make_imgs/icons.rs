use std::{
    dbg,
    fs::{self, File},
    path::PathBuf,
};

use cairo::ImageSurface;
use texture_synthesis::{Dims, ImageSource};

use crate::make_imgs;

use super::util;

pub(super) fn make_alphabet(
    min: i32,
    max: i32,
    height: i32,
    width: i32,
    font_path: &str,
    target_dir: &str,
    background_color: make_imgs::Myrgb,
    text_color: make_imgs::Myrgb,
) {
    let font = util::path_to_font(font_path);
    let _ = fs::create_dir_all(target_dir.clone());
    let base_fontsize = height as f64;
    let mut fontsize = height as f64;
    let spacing = height as f64 / 40.0;
    dbg!("making_alp", target_dir);
    for i in min..max + 1 {
        fontsize = base_fontsize;
        let s = char::from_u32(i as u32).unwrap().to_string();
        let (surface, mut context) = util::make_surface(height, width);
        context.set_font_face(&font);
        context.set_line_width(spacing);
        context = make_imgs::set_color(context, background_color);
        context.rectangle(0., 0., f64::from(width), f64::from(height));
        let _ = context.fill();
        context = make_imgs::set_color(context, text_color);
        context.set_font_size(fontsize as f64);
        let mut tx = context.text_extents(&s).unwrap();
        while tx.x_advance > 500.0 || tx.y_advance > 500.0 || tx.height > 500.0 || tx.width > 500.0
        {
            fontsize = fontsize * 0.9;
            context.set_font_size(fontsize as f64);
            tx = context.text_extents(&s).unwrap();
            //dbg!(tx);
        }
        fontsize = fontsize * 0.9;
        context.set_font_size(fontsize as f64);
        tx = context.text_extents(&s).unwrap();
        let center_x = 300.0 - (tx.width / 2.0 + tx.x_bearing);
        let center_y = 300.0 - (tx.height / 2.0 + tx.y_bearing);
        context.move_to(center_x, center_y);
        let _ = context.show_text(&s);
        let _ = context.fill();
        let mut file = File::create("".to_string() + target_dir + &i.to_string() + &".png")
            .expect("Couldn't create 'file.png'");
        surface.write_to_png(&mut file).unwrap()
    }
    dbg!("done_alp", target_dir);
}

pub(super) fn setup_alphabets(
    font_img_dir: &str,
    background_color: make_imgs::Myrgb,
    text_color: make_imgs::Myrgb,
) {
    let real_glyth_dir = "".to_string() + font_img_dir + &"real/".to_string();
    if let Ok(_x) = fs::read_dir(real_glyth_dir.clone()) {
        return;
    }
    //"fonts/Noto_Sans_Cuneiform/NotoSansCuneiform-Regular.ttf"
    // 0x12000
    // 0x1238F
    make_alphabet(
        0x11A00,
        0x11A00,
        600,
        600,
        "fonts/NotoSansZanabazarSquare-Regular.ttf",
        &real_glyth_dir,
        background_color,
        text_color,
    );
    make_alphabet(
        0x11A0B,
        0x11A32,
        600,
        600,
        "fonts/NotoSansZanabazarSquare-Regular.ttf",
        &real_glyth_dir,
        background_color,
        text_color,
    );
    make_alphabet(
        0x11A3F,
        0x11A40,
        600,
        600,
        "fonts/NotoSansZanabazarSquare-Regular.ttf",
        &real_glyth_dir,
        background_color,
        text_color,
    );
    make_alphabet(
        0x11A44,
        0x11A46,
        600,
        600,
        "fonts/NotoSansZanabazarSquare-Regular.ttf",
        &real_glyth_dir,
        background_color,
        text_color,
    );
}

pub(super) fn get_synt_glyth_path(glyth_id: i32, font_img_dir: &str) -> Option<PathBuf> {
    let synth_glyth_dir = "".to_string() + font_img_dir + &"synt/".to_string();
    let _ = fs::create_dir_all(synth_glyth_dir.clone());
    let path = synth_glyth_dir + &format!("{glyth_id}.png");
    if let Ok(_) = fs::read(&path) {
        print!("y");
        return Some(PathBuf::from(path));
    } else {
        return None;
    }
}

pub(super) fn get_real_glyth_path(seed: String, glyth_id: i32, font_img_dir: &str) -> PathBuf {
    let real_glyth_dir = "".to_string() + font_img_dir + &"real/".to_string();
    let _ = fs::create_dir_all(real_glyth_dir.clone());
    let imgs: Vec<_> = std::fs::read_dir(real_glyth_dir)
        .unwrap()
        .map(|i| i.unwrap().path())
        .collect();
    imgs[(((glyth_id + 1) * util::hash(seed)) % imgs.len() as i32).abs() as usize].clone()
}

pub(super) fn get_synth_glyth(glyth_id: i32, font_img_dir: &str) -> PathBuf {
    let real_glyth_dir = "".to_string() + font_img_dir + &"real/".to_string();
    let synth_glyth_dir = "".to_string() + font_img_dir + &"synt/".to_string();
    let _ = fs::create_dir_all(synth_glyth_dir.clone());
    let path = synth_glyth_dir + &format!("{glyth_id}.png");
    let mask = "./masks/mask600x600.png";
    let imgs: Vec<_> = std::fs::read_dir(real_glyth_dir)
        .unwrap()
        .map(|i| i.unwrap().path())
        .collect();

    let now = std::time::Instant::now();
    let mut sb = texture_synthesis::Session::builder();
    sb = sb
        .seed(glyth_id as u64)
        .resize_input(Dims {
            width: 600,
            height: 600,
        })
        .nearest_neighbors(40)
        .backtrack_stages(3)
        .backtrack_percent(0.3);
    for i in &imgs {
        if i.exists() && i.is_file() {
            sb = sb.add_example(texture_synthesis::Example::builder(i).with_guide(&mask));
        } else {
            println!("{:?}", i);
        }
    }
    let run = sb.load_target_guide(&mask).build().unwrap();
    let done = run.run(None);
    let done_img = done.into_image();

    done_img.save(path.clone()).unwrap();
    let timing = now.elapsed().as_secs();
    println!("Done: {glyth_id} Timing: {timing}");
    PathBuf::from(path)
}
