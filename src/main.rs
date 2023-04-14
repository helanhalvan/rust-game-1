pub mod actionmachine;
pub mod celldata;
pub mod css;
pub mod hexgrid;

use iced::alignment::{Horizontal, Vertical};
use iced::executor;
use iced::widget::{button, container, text};
use iced::{Application, Command, Length, Settings};
use widget::Element;

mod widget {
    use crate::css::Theme;
    //use iced::Theme;

    pub type Renderer = iced::Renderer<Theme>;
    pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
    pub type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
    pub type Button<'a, Message> = iced::widget::Button<'a, Message, Renderer>;
}

pub fn main() -> iced::Result {
    GameState::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct GameState {
    matrix: hexgrid::Board,
    resources: GameResources,
    action_machine: actionmachine::ActionMachine,
}
#[derive(Debug, Clone, Copy)]
pub struct GameResources {
    tiles: i32,
    leak: i32,
    heat_efficency: f64,
    actions: i32,
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
        let m1 = vec![vec![celldata::CellState::Hidden; 5]; 10];
        (
            GameState {
                matrix: m1,
                resources: GameResources {
                    tiles: 0,
                    leak: 1,
                    heat_efficency: 0.0,
                    actions: 10,
                    wood: 400,
                },
                action_machine: actionmachine::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Game")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Build(t, pos @ hexgrid::Pos { x, y }) => {
                self.resources.actions = self.resources.actions - 1;
                self.matrix[x][y] = t.into();
                self.action_machine =
                    actionmachine::maybe_insert(self.action_machine.clone(), pos, t);
                if let Some(new_delta) = celldata::leak_delta(t, pos, &self.matrix) {
                    self.resources.leak = self.resources.leak + new_delta;
                    self.resources.heat_efficency =
                        self.resources.tiles as f64 / self.resources.leak as f64;
                }
                if celldata::is_tile(t) {
                    self.resources.tiles = self.resources.tiles + 1;
                }
            }
            Message::EndTurn => {
                let (r1, m1) = actionmachine::run(
                    self.resources,
                    self.action_machine.clone(),
                    self.matrix.clone(),
                );
                self.resources = r1;
                self.matrix = m1;
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
                    .map(|(y_index, i)| to_gui(x_index, y_index, self.resources.actions, i.clone()))
                    .collect();
                data.insert(0, padding);
                crate::Element::from(iced::widget::Column::with_children(data))
            })
            .collect();
        let matrix = crate::Element::from(iced::widget::Row::with_children(x));
        // TODO, there should probably be a macro converting resources into something prettier
        let resources = crate::Element::from(to_text(format!("{:?}", self.resources).to_string()));
        let end_turn_content = to_text("End Turn".to_string());
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

fn to_gui<'a>(x: usize, y: usize, actions: i32, s: celldata::CellState) -> Element<'a, Message> {
    let content = match s {
        celldata::CellState::Unused => {
            if actions > 0 {
                let pos = hexgrid::Pos { x, y };
                let grid = to_rectangle(celldata::buildable(), 3, 4)
                    .iter()
                    .map(|v| {
                        crate::Element::from(iced::widget::row(
                            v.into_iter()
                                .map(|i| {
                                    let button_content =
                                        to_text(i.to_string().chars().next().unwrap().to_string());
                                    crate::Element::from(
                                        button(button_content).on_press(Message::Build(*i, pos)),
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
        celldata::CellState::Hidden => {
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
        celldata::CellState::ActionMachine(c) => to_text(format!("A {}", c).to_string()),
        celldata::CellState::Hot { slot: state, .. } => {
            to_text(format!("Hot {:?}", state).to_string())
        }
        a => {
            let v: celldata::CellStateVariant = a.into();
            to_text(v.to_string())
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

fn to_text<'a>(s: String) -> Element<'a, Message> {
    return crate::Element::from(text(s).size(20));
}
