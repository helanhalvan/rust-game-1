use core::fmt;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use iced::executor;
use iced::widget::{button, container, text};
use iced::{Application, Command, Element, Length, Settings, Theme};

pub fn main() -> iced::Result {
    GameState::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

type Board = Vec<Vec<CellState>>;

struct GameState {
    matrix: Board, // y % 2 == 0 -> down column
    tiles: i32,
    leak: i32,
    score: f64,
    actions: i32,
    action_machine: Vec<Pos>,
}

type Pos = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CellState {
    Hidden,
    Unused,
    Hot(bool),
    Insulation,
    Feeder,
    ActionMachine(i32),
}

//no data variant of CellState for easy comparision and similar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CellStateVariant {
    Hidden,
    Unused,
    Hot,
    Insulation,
    Feeder,
    ActionMachine,
}

impl Into<CellStateVariant> for CellState {
    fn into(self) -> CellStateVariant {
        match self {
            CellState::Hidden => CellStateVariant::Hidden,
            CellState::Unused => CellStateVariant::Unused,
            CellState::Hot(_) => CellStateVariant::Hot,
            CellState::Insulation => CellStateVariant::Insulation,
            CellState::Feeder => CellStateVariant::Feeder,
            CellState::ActionMachine(_) => CellStateVariant::ActionMachine,
        }
    }
}

impl Into<CellState> for CellStateVariant {
    fn into(self) -> CellState {
        match self {
            CellStateVariant::Hidden => CellState::Hidden,
            CellStateVariant::Unused => CellState::Unused,
            CellStateVariant::Hot => CellState::Hot(false),
            CellStateVariant::Insulation => CellState::Insulation,
            CellStateVariant::Feeder => CellState::Feeder,
            CellStateVariant::ActionMachine => CellState::ActionMachine(3),
        }
    }
}

fn is_action_machine(cv: CellStateVariant) -> bool {
    match cv {
        CellStateVariant::Feeder => true,
        CellStateVariant::ActionMachine => true,
        _ => false,
    }
}
fn is_tile(cv: CellStateVariant) -> bool {
    match cv {
        CellStateVariant::Hot => true,
        _ => false,
    }
}
fn buildable() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Hot,
        CellStateVariant::Insulation,
        CellStateVariant::Feeder,
        CellStateVariant::ActionMachine,
    ]
}

impl fmt::Display for CellStateVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Build(CellStateVariant, Pos),
}

impl Application for GameState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut m1 = vec![vec![CellState::Hidden; 5]; 10];
        m1[3][3] = CellStateVariant::Hot.into();
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
                if is_action_machine(t) == true {
                    self.action_machine.push(pos);
                }
                if let Some(new_delta) = leak_delta(t, pos, &self.matrix) {
                    self.leak = self.leak + new_delta;
                    self.score = self.tiles as f64 / self.leak as f64;
                }
                if is_tile(t) {
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

fn to_gui<'a>(x: usize, y: usize, actions: i32, s: CellState) -> Element<'a, Message> {
    let content = match s {
        CellState::Unused => {
            if actions > 0 {
                let pos = (x, y);
                let buttons = buildable()
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
        CellState::Hidden => {
            if actions > 0 {
                let button_text = "Explore".to_string();
                let button_content = to_text(button_text);
                let b1 = button(button_content)
                    .on_press(Message::Build(CellStateVariant::Unused, (x, y)));
                crate::Element::from(b1)
            } else {
                to_text("Hidden".to_string())
            }
        }
        CellState::ActionMachine(c) => to_text(format!("A {}", c).to_string()),
        CellState::Hot(state) => to_text(format!("Hot {state}").to_string()),
        a => {
            let v: CellStateVariant = a.into();
            to_text(v.to_string())
        }
    };
    crate::Element::from(container(content).width(100).height(100))
}

fn game_tick(s: &mut Board, action_machines: &Vec<Pos>) -> i32 {
    let ret = pos_iter_to_cells(action_machines.clone(), s);
    ret.iter()
        .map(|i| match i {
            None => 0,
            Some((x, y, CellState::ActionMachine(count))) => {
                let (new, add) = if *count == 0 { (3, 1) } else { (count - 1, 0) };
                s[*x][*y] = CellState::ActionMachine(new);
                add
            }
            Some((x, y, CellState::Feeder)) => {
                let con: Vec<(usize, usize, CellState)> =
                    get_connected(*x, *y, CellStateVariant::Hot, s)
                        .into_iter()
                        .filter(|(_x, _y, i)| match i {
                            CellState::Hot(state) => !state,
                            _ => false,
                        })
                        .collect();
                match con.get(0) {
                    Some((hx, hy, CellState::Hot(false))) => {
                        s[*hx][*hy] = CellState::Hot(true);
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

fn pos_iter_to_cells(
    pos: impl IntoIterator<Item = Pos>,
    m: &Board,
) -> Vec<Option<(usize, usize, CellState)>> {
    let ret = pos
        .into_iter()
        .map(|(x, y)| {
            let ret: (usize, usize) = match (x.try_into(), y.try_into()) {
                (Ok(x1), Ok(y1)) => (x1, y1),
                _ => (usize::MAX, usize::MAX),
            };
            ret
        })
        .map(|(x, y)| match m.get(x) {
            Some(v) => match v.get(y) {
                None => None,
                Some(&a) => Some((x, y, a)),
            },
            None => None,
        })
        .collect();
    return ret;
}

// something is wrong with the graph traverse?
fn get_connected(
    x0: usize,
    y0: usize,
    t: CellStateVariant,
    m: &Board,
) -> impl IntoIterator<Item = (usize, usize, CellState)> {
    let mut set_size = 0;
    let mut connected: HashSet<(usize, usize, CellState)> = neighbors(x0, y0, m)
        .iter()
        .filter_map(|&i| match i {
            Some((_x, _y, a)) => {
                if t == a.into() {
                    i
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    while connected.len() > set_size {
        set_size = connected.len();
        let new_connected = connected
            .iter()
            .flat_map(|(x, y, _)| neighbors(*x, *y, m))
            .filter_map(|i| match i {
                Some((_x, _y, a)) if t == a.into() => i,
                _ => None,
            })
            .collect();
        connected = connected.union(&new_connected).map(|i| i.clone()).collect();
    }
    return connected;
}

fn neighbors(x0: usize, y0: usize, m: &Board) -> Vec<Option<(usize, usize, CellState)>> {
    let x: i32 = x0.try_into().unwrap();
    let y: i32 = y0.try_into().unwrap();
    let hard_neighbors = if (x % 2) == 0 {
        [(x + 1, y + 1), (x - 1, y + 1)]
    } else {
        [(x + 1, y - 1), (x - 1, y - 1)]
    };
    let mut neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)].to_vec();
    neighbors.append(&mut hard_neighbors.to_vec());
    let pos_iter = neighbors.into_iter().map(|(x, y)| {
        let ret: (usize, usize) = match (x.try_into(), y.try_into()) {
            (Ok(x1), Ok(y1)) => (x1, y1),
            _ => (usize::MAX, usize::MAX),
        };
        ret
    });
    let ret = pos_iter_to_cells(pos_iter, m);
    return ret;
}

fn leak_delta(cv: CellStateVariant, (x, y): (usize, usize), m: &Board) -> Option<i32> {
    if let Some((base, n_effects)) = match cv {
        CellStateVariant::Insulation => Some((0, HashMap::from([(CellStateVariant::Hot, -1)]))),
        CellStateVariant::Hot => Some((
            12,
            HashMap::from([
                (CellStateVariant::Hot, -2),
                (CellStateVariant::Insulation, -1),
            ]),
        )),
        _ => None,
    } {
        let n_effects_applied: i32 = neighbors(x, y, &m)
            .iter()
            .map(|i| match i {
                Some((_, _, cc)) => {
                    let ct: CellStateVariant = (*cc).into();
                    if let Some(d) = n_effects.get(&ct) {
                        *d
                    } else {
                        0
                    }
                }
                _ => 0,
            })
            .sum();
        Some(base + n_effects_applied)
    } else {
        None
    }
}
