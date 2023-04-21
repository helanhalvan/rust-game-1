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

#[derive(Debug, Clone)]
pub struct GameState {
    matrix: hexgrid::Board,
    logistics_plane: building::Board,
    resources: GameResources,
    action_machine: actionmachine::ActionMachine,
    img_buffer: visualize_cell::ImgBuffer,
}
#[derive(Debug, Clone, Copy)]
pub struct GameResources {
    tiles: i32,
    leak: i32,
    heat_efficency: f64,
    build_points: i32,
    build_in_progress: i32,
    wood: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Build(celldata::CellStateVariant, hexgrid::Pos),
    EndTurn,
}

impl Application for GameState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = css::Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let xmax = 5;
        let ymax = 10;
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
        let g = GameState {
            matrix: m1,
            logistics_plane: building::new_plane(xmax, ymax),
            resources: GameResources {
                tiles: 0,
                leak: 1,
                heat_efficency: 0.0,
                build_points: 0,
                build_in_progress: 1,
                wood: 400,
            },
            action_machine: actionmachine::new(),
            img_buffer: visualize_cell::new_img_buffer(),
        };
        let p = hexgrid::Pos { x: 4, y: 2 };
        let cv = celldata::CellStateVariant::Hub;
        let (c, mut g1) = building::finalize_build(cv, p, g);
        hexgrid::set(p, c, &mut g1.matrix);
        (g1, Command::none())
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
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let x = self
            .matrix
            .iter()
            .enumerate()
            .map(|(x_index, i)| {
                let padding: Element<'static, Message> =
                    crate::Element::from(if x_index % 2 == 0 {
                        container("").width(100).height(50)
                    } else {
                        container("").width(10).height(10)
                    });
                let mut data: Vec<Element<'static, Message>> = i
                    .iter()
                    .enumerate()
                    .map(|(y_index, i)| {
                        visualize_cell::to_gui(
                            x_index,
                            y_index,
                            building::has_actions(
                                hexgrid::Pos {
                                    x: x_index,
                                    y: y_index,
                                },
                                self,
                            ),
                            i.clone(),
                            &self.img_buffer,
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
        let content = iced::widget::Column::with_children(vec![matrix, resources, buttom_buttons]);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}
