use image_generator::{objects::Object, structure::Structure};

use crate::{actionmachine, celldata};

pub fn make_imgs() {
    let image_variants = [
        celldata::CellStateVariant::ActionMachine,
        celldata::CellStateVariant::Hot,
    ];
    let base = Structure::load_from_file("./img_gen/base.json").unwrap();
    for i in image_variants {
        let series = base.clone();
        let theme = image_generator::structure::ImageContext::new(&series);
        let max_size = actionmachine::in_progress_max(i);
        for j in 1..max_size + 1 {
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

fn make_path(cv: celldata::CellStateVariant, i: u32) -> String {
    "./img/".to_string() + &cv.to_string() + &i.to_string() + &".png"
}
