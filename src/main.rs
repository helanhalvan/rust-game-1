use iced::executor;
use iced::mouse::Button;
use iced::widget::canvas::{stroke, Cache, Cursor, Geometry, LineCap, Path, Stroke};
use iced::widget::{button, column, container, text, Column, Text};
use iced::{
    Application, Color, Command, Element, Length, Point, Rectangle, Settings, Subscription, Theme,
    Vector,
};

pub fn main() -> iced::Result {
    GameState::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct GameState {
    matrix: Vec<Vec<CellState>>,
}

#[derive(Debug, Clone, Copy)]
struct Hidden {
    marked: bool,
    is_mine: bool,
}

#[derive(Debug, Clone, Copy)]
enum CellState {
    Hidden(Hidden),
    Hint(i32),
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Mark(usize, usize),
    UnMark(usize, usize),
    Explore(usize, usize),
}

impl Application for GameState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let m1 = vec![
            vec![
                CellState::Hidden(Hidden {
                    marked: false,
                    is_mine: false
                });
                10
            ];
            5
        ];
        // TODO proper place out mines
        let mut x: Vec<Vec<CellState>> = m1
            .iter()
            .map(|i| i.iter().map(|i| i.clone()).collect())
            .collect();
        x[0][0] = CellState::Hidden(Hidden {
            marked: false,
            is_mine: true,
        });
        (GameState { matrix: x }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Game")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Explore(x0, y0) => match &self.matrix[x0][y0] {
                CellState::Hidden(s @ Hidden { is_mine: false, .. }) => {
                    let x: i32 = x0.try_into().unwrap();
                    let y: i32 = y0.try_into().unwrap();
                    let s = sum_mines(
                        self.matrix.clone(),
                        [
                            (x + 1, y),
                            (x - 1, y),
                            (x, y + 1),
                            (x, y - 1),
                            (x + 1, y + 1),
                            (x + 1, y - 1),
                            (x - 1, y + 1),
                            (x - 1, y - 1),
                        ]
                        .to_vec(),
                    );
                    self.matrix[x0][y0] = CellState::Hint(s);
                }
                CellState::Hidden(s @ Hidden { is_mine: true, .. }) => {
                    self.matrix[x0][y0] = CellState::Hint(9999);
                }
                s => {
                    println!("Bad Cell{:#?}\n", s);
                    unimplemented!()
                }
            },
            Message::Mark(x, y) => match &self.matrix[x][y] {
                CellState::Hidden(s @ Hidden { marked: false, .. }) => {
                    self.matrix[x][y] = CellState::Hidden(Hidden {
                        marked: true,
                        ..s.clone()
                    })
                }
                s => {
                    println!("Bad Cell{:#?}\n", s);
                    unimplemented!()
                }
            },
            Message::UnMark(x, y) => match &self.matrix[x][y] {
                CellState::Hidden(s @ Hidden { marked: true, .. }) => {
                    self.matrix[x][y] = CellState::Hidden(Hidden {
                        marked: false,
                        ..s.clone()
                    })
                }
                s => {
                    println!("Bad Cell{:#?}\n", s);
                    unimplemented!()
                }
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let x = self
            .matrix
            .iter()
            .enumerate()
            .map(|(y_index, i)| {
                crate::Element::from(iced::widget::Row::with_children(
                    i.iter()
                        .enumerate()
                        .map(|(x_index, i)| to_gui(y_index, x_index, i.clone()))
                        .collect(),
                ))
            })
            .collect();
        let content = iced::widget::Column::with_children(x);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}

fn to_gui(y: usize, x: usize, s: CellState) -> Element<'static, Message> {
    let content = match s {
        CellState::Hidden(Hidden { marked: false, .. }) => {
            let button_text = "mark".to_string();
            let button_content = to_text(button_text);
            let b1 = button(button_content).on_press(Message::Mark(y, x));
            let button_text = "explore".to_string();
            let button_content = to_text(button_text);
            let b2 = button(button_content).on_press(Message::Explore(y, x));
            crate::Element::from(iced::widget::column!(b1, b2))
        }
        CellState::Hidden(Hidden { marked: true, .. }) => {
            let button_text = "unmark".to_string();
            let button_content = to_text(button_text);
            let b1 = button(button_content).on_press(Message::UnMark(y, x));
            crate::Element::from(b1)
        }
        CellState::Hint(i) => {
            let text = format!("{}", i);
            to_text(text)
        }
    };
    crate::Element::from(container(content).width(100).height(100))
}

fn to_text(s: String) -> Element<'static, Message> {
    crate::Element::from(text(s).size(20))
}

fn sum_mines(m: Vec<Vec<CellState>>, cells: Vec<(i32, i32)>) -> i32 {
    let ret = cells
        .into_iter()
        .filter_map(|(x, y)| {
            let ret: Option<(usize, usize)> = match (x.try_into(), y.try_into()) {
                (Ok(x1), Ok(y1)) => Some((x1, y1)),
                _ => None,
            };
            ret
        })
        .map(|(x, y)| match m.get(x) {
            Some(v) => match v.get(y) {
                Some(CellState::Hidden(Hidden{ is_mine: true, ..})) => 1,
                _ => 0,
            },
            None => 0,
        })
        .sum();
    ret
}
