use std::collections::HashMap;

use crate::{
    celldata::{self, Slot},
    css, hexgrid, make_imgs, widget, Message,
};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, image, text},
};
use widget::Element;

pub type ImgBuffer = HashMap<celldata::CellState, image::Handle>;

pub fn new_img_buffer() -> ImgBuffer {
    let ret = make_imgs::all_imgs()
        .iter()
        .map(|(i, j)| (*i, image::Handle::from_path(j)))
        .collect();
    return ret;
}

fn has_image(s: celldata::CellState, buff: &ImgBuffer) -> Option<&image::Handle> {
    buff.get(&s)
}

pub fn to_gui<'a>(
    x: usize,
    y: usize,
    actions: i32,
    s: celldata::CellState,
    imgs: &ImgBuffer,
) -> Element<'a, Message> {
    let content = match has_image(s, imgs) {
        Some(img) => crate::Element::from(image::viewer(img.clone())),
        None => match s {
            celldata::CellState::Unit {
                variant: celldata::CellStateVariant::Unused,
            } => {
                if actions > 0 {
                    let pos = hexgrid::Pos { x, y };
                    let grid = to_rectangle(celldata::buildable(), 3, 4)
                        .iter()
                        .map(|v| {
                            crate::Element::from(iced::widget::row(
                                v.into_iter()
                                    .map(|i| {
                                        let button_content = to_text(
                                            i.to_string().chars().next().unwrap().to_string(),
                                        );
                                        crate::Element::from(
                                            button(button_content)
                                                .on_press(Message::Build(*i, pos)),
                                        )
                                    })
                                    .collect(),
                            ))
                        })
                        .collect();
                    crate::Element::from(iced::widget::column(grid))
                } else {
                    to_text("Unused".to_string())
                }
            }
            celldata::CellState::Unit {
                variant: celldata::CellStateVariant::Hidden,
            } => {
                if actions > 0 {
                    let button_text = "Explore".to_string();
                    let button_content = to_text(button_text);
                    let b1 = button(button_content).on_press(Message::Build(
                        celldata::CellStateVariant::Unused,
                        hexgrid::Pos { x, y },
                    ));
                    crate::Element::from(b1)
                } else {
                    to_text("Hidden".to_string())
                }
            }
            /*
            celldata::CellState::Hot { slot: state, .. } => {
                to_text(format!("Hot {:?}", state).to_string())
            }
            */
            a => {
                let v: celldata::CellStateVariant = a.into();
                to_text(v.to_string())
            }
        },
    };
    crate::Element::from(
        container(content)
            .width(100)
            .height(100)
            .style(css::Container::Bordered)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
    )
}

// example, w=2, h=3, s=[1,2,3,4,5]
// |1,2|
// |3,4|
// |5|
pub fn to_rectangle<T: Clone>(
    source: impl IntoIterator<Item = T>,
    height: usize,
    width: usize,
) -> Vec<Vec<T>> {
    let (_, ret) = source.into_iter().fold(
        (0, vec![Vec::new(); height]),
        |(mut next_empty, mut a), b| {
            if a[next_empty].len() == width {
                next_empty = next_empty + 1;
            }
            a[next_empty].push(b);
            (next_empty, a)
        },
    );
    ret
}

pub fn to_text<'a>(s: String) -> Element<'a, Message> {
    return crate::Element::from(text(s).size(20));
}
