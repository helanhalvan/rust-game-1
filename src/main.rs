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
    tiles: i32,
    leak: i32,
    score: f64,
    actions: i32,
    action_machine: Vec<hexgrid::Pos>,
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
                tiles: 1,
                leak: 12,
                score: 1.0 / 12.0,
                actions: 10000,
                action_machine: vec![],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Game")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Build(t, pos @ (x, y)) => {
                self.matrix[x][y] = t.into();
                if celldata::is_action_machine(t) == true {
                    self.action_machine.push(pos);
                }
                if let Some(new_delta) = celldata::leak_delta(t, pos, &self.matrix) {
                    self.leak = self.leak + new_delta;
                    self.score = self.tiles as f64 / self.leak as f64;
                }
                if celldata::is_tile(t) {
                    self.tiles = self.tiles + 1;
                }
            }
        }
        self.actions = self.actions + game_tick(&mut self.matrix, &self.action_machine) - 1;
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
                    .map(|(y_index, i)| to_gui(x_index, y_index, self.actions, i.clone()))
                    .collect();
                data.insert(0, padding);
                crate::Element::from(iced::widget::Column::with_children(data))
            })
            .collect();
        let matrix = crate::Element::from(iced::widget::Row::with_children(x));
        // TODO some standard way of handling gamestate numbers that needs displaying
        let tiles_e = to_text(format!("Tiles: {} ", self.tiles).to_string());
        let leak_e = to_text(format!("Leak: {} ", self.leak).to_string());
        let score_e = to_text(format!("Score: {}", self.score).to_string());
        let actions_e = to_text(format!("Actions: {}", self.actions).to_string());
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
                let pos = (x, y);
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
                let b1 = button(button_content)
                    .on_press(Message::Build(celldata::CellStateVariant::Unused, (x, y)));
                crate::Element::from(b1)
            } else {
                to_text("Hidden".to_string())
            }
        }
        celldata::CellState::ActionMachine(c) => to_text(format!("A {}", c).to_string()),
        celldata::CellState::Hot(state) => to_text(format!("Hot {state}").to_string()),
        a => {
            let v: celldata::CellStateVariant = a.into();
            to_text(v.to_string())
        }
    };
    crate::Element::from(container(content).width(100).height(100))
}

fn game_tick(s: &mut hexgrid::Board, action_machines: &Vec<hexgrid::Pos>) -> i32 {
    let ret = hexgrid::pos_iter_to_cells(action_machines.clone(), s);
    ret.iter()
        .map(|i| match i {
            None => 0,
            Some((x, y, celldata::CellState::ActionMachine(count))) => {
                let (new, add) = if *count == 0 { (3, 1) } else { (count - 1, 0) };
                s[*x][*y] = celldata::CellState::ActionMachine(new);
                add
            }
            Some((x, y, celldata::CellState::Feeder)) => {
                let con: Vec<(usize, usize, celldata::CellState)> =
                    hexgrid::get_connected(*x, *y, celldata::CellStateVariant::Hot, s)
                        .into_iter()
                        .filter(|(_x, _y, i)| match i {
                            celldata::CellState::Hot(state) => !state,
                            _ => false,
                        })
                        .collect();
                match con.get(0) {
                    Some((hx, hy, celldata::CellState::Hot(false))) => {
                        s[*hx][*hy] = celldata::CellState::Hot(true);
                    }
                    _ => {}
                }
                0
            }
            Some((x, y, a)) => {
                println!("unexpected {:?}{:?}{:?}", x, y, a);
                unimplemented!()
            }
        })
        .sum()
}

fn to_text<'a>(s: String) -> Element<'a, Message> {
    return crate::Element::from(text(s).size(20));
}
