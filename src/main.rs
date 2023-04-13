pub mod actionmachine;
pub mod celldata;
pub mod hexgrid;

use iced::executor;
use iced::widget::{button, container, text};
use iced::{Application, Command, Element, Length, Settings, Theme};

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
    score: f64,
    actions: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Build(celldata::CellStateVariant, hexgrid::Pos),
}

impl Application for GameState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut m1 = vec![vec![celldata::CellState::Hidden; 5]; 10];
        m1[3][3] = celldata::CellStateVariant::Hot.into();
        (
            GameState {
                matrix: m1,
                resources: GameResources {
                    tiles: 1,
                    leak: 12,
                    score: 1.0 / 12.0,
                    actions: 10000,
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
        let (r1, m1) = actionmachine::run(
            self.resources,
            self.action_machine.clone(),
            self.matrix.clone(),
        );
        self.resources = r1;
        self.matrix = m1;
        //         self.resources.actions =
        //self.resources.actions + game_tick(&mut self.matrix, &self.action_machine) - 1;

        match message {
            Message::Build(t, pos @ hexgrid::Pos { x, y }) => {
                self.matrix[x][y] = t.into();
                self.action_machine =
                    actionmachine::maybe_insert(self.action_machine.clone(), pos, t);
                if let Some(new_delta) = celldata::leak_delta(t, pos, &self.matrix) {
                    self.resources.leak = self.resources.leak + new_delta;
                    self.resources.score = self.resources.tiles as f64 / self.resources.leak as f64;
                }
                if celldata::is_tile(t) {
                    self.resources.tiles = self.resources.tiles + 1;
                }
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
        // TODO some standard way of handling gamestate numbers that needs displaying
        let tiles_e = to_text(format!("Tiles: {} ", self.resources.tiles).to_string());
        let leak_e = to_text(format!("Leak: {} ", self.resources.leak).to_string());
        let score_e = to_text(format!("Score: {}", self.resources.score).to_string());
        let actions_e = to_text(format!("Actions: {}", self.resources.actions).to_string());
        let score = crate::Element::from(iced::widget::Row::with_children(vec![
            tiles_e, leak_e, score_e, actions_e,
        ]));
        let content = iced::widget::Column::with_children(vec![matrix, score]);

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
                let buttons = celldata::buildable()
                    .into_iter()
                    .map(|i| {
                        let button_content =
                            to_text(i.to_string().chars().next().unwrap().to_string()); //first char of string
                        crate::Element::from(
                            button(button_content).on_press(Message::Build(i, pos)),
                        )
                    })
                    .collect();
                crate::Element::from(iced::widget::row(buttons))
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
    crate::Element::from(container(content).width(100).height(100))
}

fn to_text<'a>(s: String) -> Element<'a, Message> {
    return crate::Element::from(text(s).size(20));
}
