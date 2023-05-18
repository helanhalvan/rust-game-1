use cairo::{freetype::freetype::FT_FaceRec_, FontFace, ImageSurface};
use serde::Serialize;
use std::hash::Hash;
use std::{collections::hash_map::DefaultHasher, ffi::c_void, fs, hash::Hasher, rc::Rc};

pub(super) fn path_to_font(path: &str) -> FontFace {
    let lib = freetype::Library::init().unwrap();
    let face: freetype::face::Face = lib.new_face(path, 0).unwrap();
    let cairo_face = create_from_ft(&face);
    cairo_face
}

//copied from newer cairo version
static FT_FACE_KEY: cairo::UserDataKey<freetype::face::Face> = cairo::UserDataKey::new();
fn create_from_ft(face: &freetype::face::Face) -> FontFace {
    let mut face = face.clone();
    let font_face = unsafe {
        FontFace::from_raw_full(cairo::ffi::cairo_ft_font_face_create_for_ft_face(
            face.raw_mut() as freetype::ffi::FT_Face as *mut _,
            0,
        ))
    };
    font_face
        .set_user_data(&FT_FACE_KEY, Rc::new(face))
        .unwrap();

    font_face
}

pub(super) fn make_surface(height: i32, width: i32) -> (ImageSurface, cairo::Context) {
    let surface = cairo::ImageSurface::create(cairo::Format::Rgb24.into(), width, height).unwrap();
    let context = cairo::Context::new(&surface).unwrap();
    (surface, context)
}

pub(super) fn get_with_file_cache<'a, T, S: serde::de::DeserializeOwned + Serialize + Clone>(
    base: &str,
    name: &str,
    parse: fn(S) -> T,
    make: fn() -> S,
) -> T {
    fs::create_dir_all(base.clone()).unwrap();
    let path = "".to_string() + base + name;
    if let Ok(s0) = fs::read_to_string(&path) {
        let s: S = serde_json::from_str::<S>(&s0).unwrap();
        parse(s)
    } else {
        let new = make();
        let s = serde_json::to_string(&new).unwrap();
        fs::write(path, s).unwrap();
        parse(new)
    }
}

pub(super) fn hash<T: Hash>(seed: T) -> i32 {
    let mut s = DefaultHasher::new();
    seed.hash(&mut s);
    let u = s.finish();
    u as i32
}
