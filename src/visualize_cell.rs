use std::collections::HashMap;

use crate::{
    celldata::{self},
    css::{self},
    hexgrid, menu, widget, GameState, Message,
};
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, image, text},
};
use iced_native::Length;
use widget::Element;

pub(crate) type ImgBuffer = HashMap<celldata::CellState, image::Handle>;

pub(crate) fn new_img_buffer() -> ImgBuffer {
    return HashMap::new();
}

fn has_image(s: celldata::CellState, buff: &ImgBuffer) -> Option<&image::Handle> {
    buff.get(&s)
}

pub(crate) const START_CELL_X_SIZE: f32 = 100.0;
pub(crate) const START_CELL_Y_SIZE: f32 = 125.0;
pub(crate) const ZOOM_FACTOR: f32 = 1.5;

pub(crate) fn to_gui<'a>(
    pos: hexgrid::XYCont<i32>,
    s: celldata::CellState,
    g: &GameState,
    send: &std::sync::mpsc::Sender<celldata::CellState>,
) -> Element<'a, Message> {
    let imgs: &ImgBuffer = &g.img_buffer;
    let content = match menu::has_actions(pos, s, g) {
        Some(actions) => render_action_cell(actions, pos, imgs, s, send),
        None => match has_image(s, imgs) {
            Some(img_handle) => to_image(img_handle),
            None => {
                let _ = send.send(s);
                backup_formatter(s)
            }
        },
    };

    crate::Element::from(
        container(content)
            .width(g.io_cache.cell_y_size)
            .height(g.io_cache.cell_x_size)
            .style(css::Container::Bordered)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),
    )
}

fn render_action_cell<'a>(
    actions: Vec<celldata::CellStateVariant>,
    pos: hexgrid::Pos,
    imgs: &ImgBuffer,
    s: celldata::CellState,
    send: &std::sync::mpsc::Sender<celldata::CellState>,
) -> Element<'a, Message> {
    let layout = if actions.len() > 2 {
        to_rectangle(actions, 4, 2)
    } else {
        to_rectangle(actions, 2, 1)
    };
    let mut grid: Vec<_> = layout
        .iter()
        .map(|v| {
            crate::Element::from(iced::widget::row(
                v.into_iter()
                    .map(|i| {
                        let button_content = to_text(i.to_string());
                        crate::Element::from(
                            button(button_content).on_press(Message::Build(*i, pos)),
                        )
                    })
                    .collect(),
            ))
        })
        .collect();
    let top = match has_image(s, imgs) {
        Some(img) => to_image(img),
        None => {
            let _ = send.send(s);
            to_text(format!("{:?}", s.variant).to_string())
        }
    };
    grid.push(top);
    crate::Element::from(iced::widget::column(grid))
}

fn backup_formatter<'a>(s: celldata::CellState) -> Element<'a, Message> {
    //dbg!(s);
    match s.data {
        celldata::CellStateData::Unit => to_text(format!("DEFAULT:{:?}", s.variant).to_string()),
        _ => to_text(format!("DEFAULT:{:?}", s).to_string()),
    }
}

// example, w=2, h=3, s=[1,2,3,4,5]
// |1,2|
// |3,4|
// |5|
pub(crate) fn to_rectangle<T: Clone>(
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

pub(crate) fn to_text<'a>(s: String) -> Element<'a, Message> {
    return crate::Element::from(text(s).size(20));
}

pub(crate) fn to_image<'a>(img_handle: &image::Handle) -> Element<'a, Message> {
    let image = iced::widget::Image::new(img_handle.clone())
        .width(Length::Fill)
        .height(Length::Fill);
    crate::Element::from(container(image))
}
