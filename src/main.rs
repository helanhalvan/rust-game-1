pub mod actionmachine;
pub mod building;
pub mod celldata;
pub mod css;
pub mod hexgrid;
pub mod make_imgs;
pub mod visualize_cell;

use iced::executor;
use iced::widget::{button, container};
use iced::{Application, Command, Length, Settings};
use iced_native::row;
use widget::Element;

mod widget {
    use crate::css::Theme;
    //use iced::Theme;

    pub type Renderer = iced::Renderer<Theme>;
    pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
    //pub type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
    //pub type Button<'a, Message> = iced::widget::Button<'a, Message, Renderer>;
}

use std::{env, vec};

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let _ = GameState::run(Settings {
            antialiasing: true,
            ..Settings::default()
        });
    } else if args[1] == "images" {
        make_imgs::make_imgs();
    } else {
        dbg!(args);
    }
}

pub type WindowPos = iced_native::Point;

#[derive(Debug, Clone)]
pub struct GameState {
    matrix: hexgrid::Board,
    logistics_plane: building::LogisticsPlane,
    resources: GameResources,
    action_machine: actionmachine::ActionMachine,
    img_buffer: visualize_cell::ImgBuffer,
    top_left_pos: iced_native::Point,
    latest_cursor: iced_native::Point,
    is_mousedown: bool,
    top_left_hex: hexgrid::XYCont<i32>,
}
#[derive(Debug, Clone, Copy)]
pub struct GameResources {
    tiles: i32,
    leak: i32,
    heat_efficency: f64,
    wood: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Build(celldata::CellStateVariant, hexgrid::Pos),
    EndTurn,
    NativeEvent(iced_native::Event),
}

impl Application for GameState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = css::Theme;
    type Flags = ();

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced_native::subscription::events().map(Message::NativeEvent)
    }

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let xmax = 50;
        let ymax = 90;
        let start_x: usize = 25;
        let start_y: usize = 45;
        let m1 = vec![
            vec![
                celldata::CellState {
                    variant: celldata::CellStateVariant::Hidden,
                    data: celldata::CellStateData::Unit
                };
                xmax
            ];
            ymax
        ];
        let mut g = GameState {
            matrix: m1,
            logistics_plane: building::new_plane(xmax, ymax),
            resources: GameResources {
                tiles: 0,
                leak: 1,
                heat_efficency: 0.0,
                wood: 400,
            },
            action_machine: actionmachine::new(),
            img_buffer: visualize_cell::new_img_buffer(),
            top_left_pos: iced::Point {
                x: (start_x as f32 - (visualize_cell::VIEW_CELLS_X / 2) as f32)
                    * visualize_cell::CELL_X_SIZE,
                y: (start_y as f32 - (visualize_cell::VIEW_CELLS_Y / 2) as f32)
                    * visualize_cell::CELL_Y_SIZE,
            },
            latest_cursor: iced::Point { x: 0.0, y: 0.0 },
            is_mousedown: false,
            top_left_hex: hexgrid::XYCont {
                x: start_x as i32 - (visualize_cell::VIEW_CELLS_X / 2),
                y: start_y as i32 - (visualize_cell::VIEW_CELLS_Y / 2),
            },
        };
        let p = hexgrid::Pos {
            x: start_x,
            y: start_y,
        };
        let cv = celldata::CellStateVariant::Hub;
        g = building::do_build(cv, p, g);
        (g, Command::none())
    }

    fn title(&self) -> String {
        String::from("Game")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Build(t, pos) => *self = building::build(t, pos, self.clone()),
            Message::EndTurn => {
                *self = actionmachine::run(self.clone());
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::CursorMoved {
                position,
            })) => {
                if (*self).is_mousedown == true {
                    let old_p = (*self).latest_cursor;
                    let delta = old_p - position;
                    (*self).top_left_pos = (*self).top_left_pos + delta;
                    (*self).top_left_hex = approx((*self).top_left_pos);
                }
                (*self).latest_cursor = position;
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            ))) => {
                (*self).is_mousedown = true;
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            ))) => {
                (*self).is_mousedown = false;
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::CursorLeft)) => {
                (*self).is_mousedown = false;
            }
            Message::NativeEvent(_) => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let view_matrix = hexgrid::sub_matrix(
            &self.matrix,
            self.top_left_hex,
            visualize_cell::VIEW_CELLS_X - 1,
            visualize_cell::VIEW_CELLS_Y - 1,
            celldata::unit_state(celldata::CellStateVariant::OutOfBounds),
        );
        let hexgrid::XYCont {
            x: base_x,
            y: base_y,
        } = self.top_left_hex;
        let x = view_matrix
            .iter()
            .enumerate()
            .map(|(x_index, i)| {
                let padding: Element<'static, Message> =
                    crate::Element::from(if (base_x + x_index as i32) % 2 == 0 {
                        container("")
                            .width(visualize_cell::CELL_Y_SIZE)
                            .height(visualize_cell::CELL_X_SIZE / 2.0)
                    } else {
                        container("").width(10).height(10)
                    });
                let mut data: Vec<Element<'static, Message>> = i
                    .iter()
                    .enumerate()
                    .map(|(y_index, i)| {
                        let yet_another_x: i32 = x_index.try_into().unwrap();
                        let yet_another_y: i32 = y_index.try_into().unwrap();
                        let matrix_x: i32 = base_x + yet_another_x;
                        let matrix_y: i32 = base_y + yet_another_y;
                        visualize_cell::to_gui(
                            hexgrid::XYCont {
                                x: matrix_x,
                                y: matrix_y,
                            },
                            i.clone(),
                            &self,
                        )
                    })
                    .collect();
                data.insert(0, padding);
                crate::Element::from(iced::widget::Column::with_children(data))
            })
            .collect();
        let matrix = crate::Element::from(iced::widget::Row::with_children(x));
        let resources = crate::Element::from(visualize_cell::to_text(
            format!("{:?}", self.resources).to_string(),
        ));
        let end_turn_content = visualize_cell::to_text("End Turn".to_string());
        let buttom_buttons =
            crate::Element::from(button(end_turn_content).on_press(Message::EndTurn));
        let ui_misc = crate::Element::from(row![
            visualize_cell::to_text(format!("{:?}", self.top_left_pos).to_string()),
            visualize_cell::to_text(format!("{:?}", self.latest_cursor).to_string()),
        ]);
        let content =
            iced::widget::Column::with_children(vec![matrix, resources, buttom_buttons, ui_misc]);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}

fn approx(iced::Point { x, y }: iced_native::Point) -> hexgrid::XYCont<i32> {
    hexgrid::XYCont {
        x: (x / visualize_cell::CELL_X_SIZE) as i32,
        y: (y / visualize_cell::CELL_Y_SIZE) as i32,
    }
}
