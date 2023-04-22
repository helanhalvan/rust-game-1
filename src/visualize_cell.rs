use std::collections::HashMap;

use crate::{
    building,
    celldata::{self},
    css, hexgrid, make_imgs, widget, GameState, Message,
};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, image, row, text},
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
    raw_pos: hexgrid::XYCont<i32>,
    s: celldata::CellState,
    g: &GameState,
) -> Element<'a, Message> {
    let imgs: &ImgBuffer = &g.img_buffer;
    let content = match has_image(s, imgs) {
        Some(img) => {
            let v: celldata::CellStateVariant = s.into();
            crate::Element::from(iced::widget::column(vec![
                to_text(v.to_string()),
                crate::Element::from(image::viewer(img.clone())),
            ]))
        }
        None => {
            if let Some((pos, _)) = hexgrid::to_pos_cell(raw_pos, &g.matrix) {
                match (s, building::has_actions(pos, g)) {
                    (
                        celldata::CellState {
                            variant: celldata::CellStateVariant::Unused,
                            ..
                        },
                        true,
                    ) => {
                        let grid = to_rectangle(building::buildable(), 3, 4)
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
                    }
                    (
                        celldata::CellState {
                            variant: celldata::CellStateVariant::Hidden,
                            ..
                        },
                        true,
                    ) => {
                        let button_text = "Explore".to_string();
                        let button_content = to_text(button_text);
                        let b1 = button(button_content)
                            .on_press(Message::Build(celldata::CellStateVariant::Unused, pos));
                        crate::Element::from(b1)
                    }
                    (a, _) => {
                        dbg!(a);
                        backup_formatter(s)
                    }
                }
            } else {
                backup_formatter(s)
            }
        }
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

fn backup_formatter<'a>(s: celldata::CellState) -> Element<'a, Message> {
    match s.data {
        celldata::CellStateData::Unit => to_text(format!("{:?}", s.variant).to_string()),
        _ => to_text(format!("{:?}", s).to_string()),
    }
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
