use std::fs;

use iced::theme;
use image_generator::{
    objects::Object,
    structure::{ImageContext, Structure},
};

use crate::celldata::{self, non_interactive_statespace, CellStateVariant};

pub fn all_imgs() -> Vec<(celldata::CellState, String)> {
    let mut all_imgs_buff = Vec::new();
    for (_cv, variant_space) in non_interactive_statespace() {
        for c in variant_space {
            all_imgs_buff.push((c, make_path(c)))
        }
    }
    return all_imgs_buff;
}

pub fn make_imgs() {
    let base = Structure::load_from_file("./img_gen/base.json").unwrap();
    for (_cv, variant_space) in non_interactive_statespace() {
        let series = base.clone();
        let theme = image_generator::structure::ImageContext::new(&series);
        for cellstate in variant_space {
            let path = make_path(cellstate);
            let (obj_count, obj_name) = match cellstate {
                celldata::CellState::InProgress { countdown, .. } => {
                    (countdown, "inprogress".to_string())
                }

                celldata::CellState::Unit { .. } => (1, "inprogress".to_string()),
                celldata::CellState::Slot {
                    slot: celldata::Slot::Done,
                    ..
                } => (1, "done".to_string()),
                celldata::CellState::Slot {
                    slot: celldata::Slot::Empty,
                    ..
                } => (1, "empty".to_string()),
            };

            let mut img_base = theme.clone();

            let mut newobj = img_base.objects.clone();
            match img_base.objects.get("main") {
                Some(Object::Sun(s)) => {
                    let mut s2 = s.clone();
                    s2.segments = obj_count.try_into().unwrap();
                    s2.query = image_generator::structure::Query::ByName {
                        by_name: obj_name,
                        choose: image_generator::structure::Choose::Once,
                    };
                    newobj.insert("main".to_string(), Object::Sun(s2));
                    img_base.objects = &newobj;
                }
                _ => unimplemented!(),
            };
            image_generator::stable_color_entrypoint(base.clone(), path, img_base.clone());
        }
    }
}

fn make_path(cellstate: celldata::CellState) -> String {
    match cellstate {
        celldata::CellState::InProgress {
            variant: cv,
            countdown,
            ..
        } => {
            let dir = "./img/".to_string() + &cv.to_string();
            let _ = fs::create_dir_all(dir.clone());
            dir + "inprogress_" + &countdown.to_string() + &".png"
        }
        celldata::CellState::Unit { variant: cv } => {
            let dir = "./img/".to_string() + &cv.to_string();
            let _ = fs::create_dir_all(dir.clone());
            dir + "unit" + &".png"
        }
        celldata::CellState::Slot {
            variant: cv,
            slot: celldata::Slot::Empty,
        } => {
            let dir = "./img/".to_string() + &cv.to_string();
            let _ = fs::create_dir_all(dir.clone());
            dir + "empty" + &".png"
        }
        celldata::CellState::Slot {
            variant: cv,
            slot: celldata::Slot::Done,
        } => {
            let dir = "./img/".to_string() + &cv.to_string();
            let _ = fs::create_dir_all(dir.clone());
            dir + "done" + &".png"
        }
    }
}
