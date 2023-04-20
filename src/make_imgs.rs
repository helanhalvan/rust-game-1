use std::fs;

use image_generator::{objects::Object, structure::Structure};

use crate::{
    actionmachine,
    celldata::{self, CellStateVariant},
};

pub fn all_imgs() -> Vec<(celldata::CellState, String)> {
    let mut all_imgs_buff = Vec::new();
    for i in image_variants() {
        let max_size = actionmachine::in_progress_max(i);
        for j in 1..max_size + 2 {
            all_imgs_buff.push((
                celldata::CellState::InProgress {
                    variant: i,
                    countdown: j,
                },
                make_path(i, j),
            ))
        }
    }
    return all_imgs_buff;
}

pub fn make_imgs() {
    let base = Structure::load_from_file("./img_gen/base.json").unwrap();
    for i in image_variants() {
        let series = base.clone();
        let theme = image_generator::structure::ImageContext::new(&series);
        let max_size = actionmachine::in_progress_max(i);
        for j in 1..max_size + 2 {
            let mut img_base = theme.clone();
            let mut newobj = img_base.objects.clone();
            match img_base.objects.get("main") {
                Some(Object::Sun(s)) => {
                    let mut s2 = s.clone();
                    s2.segments = j.try_into().unwrap();
                    newobj.insert("main".to_string(), Object::Sun(s2));
                    img_base.objects = &newobj;
                }
                _ => unimplemented!(),
            };
            let path = make_path(i, j);
            image_generator::stable_color_entrypoint(base.clone(), path, img_base);
        }
    }
}

fn image_variants() -> [CellStateVariant; 2] {
    [
        celldata::CellStateVariant::ActionMachine,
        celldata::CellStateVariant::Hot,
    ]
}

fn make_path(cv: celldata::CellStateVariant, i: u32) -> String {
    let dir = "./img/".to_string() + &cv.to_string();
    let _ = fs::create_dir_all(dir.clone());
    dir + "/" + &i.to_string() + &".png"
}
