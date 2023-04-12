use iced::executor;
use iced::futures::SinkExt;
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
    matrix: Vec<Vec<CellState>>, // y % 2 == 0 -> down column
    tiles: i32,
    leak: i32,
    score: f64,
}

#[derive(Debug, Clone, Copy)]
struct Hidden {
    marked: bool,
    is_mine: bool,
}

#[derive(Debug, Clone, Copy)]
enum CellState {
    Unused,
    Hot,
    Insulation,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Heat(usize, usize),
    Insulate(usize, usize),
    //Explore(usize, usize),
}

impl Application for GameState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut m1 = vec![vec![CellState::Unused; 5]; 10];
        m1[3][3] = CellState::Hot;
        (
            GameState {
                matrix: m1,
                tiles: 1,
                leak: 12,
                score: 1.0 / 12.0,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Game")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Heat(x, y) => {
                self.tiles = self.tiles + 1;
                self.matrix[x][y] = CellState::Hot;
                self.leak = self.leak + leak_delta(x, y, &self.matrix);
                self.score = self.tiles as f64 / self.leak as f64;
            }
            Message::Insulate(x, y) => {
                self.matrix[x][y] = CellState::Insulation;
                self.leak = self.leak + leak_delta_ins(x, y, &self.matrix);
                self.score = self.tiles as f64 / self.leak as f64;
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let x = self
            .matrix
            .iter()
            .enumerate()
            .map(|(y_index, i)| {
                let padding: Element<'static, Message> =
                    crate::Element::from(if (y_index % 2 == 0) {
                        container("").width(100).height(50)
                    } else {
                        container("").width(10).height(10)
                    });
                let mut data: Vec<Element<'static, Message>> = i
                    .iter()
                    .enumerate()
                    .map(|(x_index, i)| to_gui(y_index, x_index, i.clone()))
                    .collect();
                data.insert(0, padding);
                crate::Element::from(iced::widget::Column::with_children(data))
            })
            .collect();
        let matrix = crate::Element::from(iced::widget::Row::with_children(x));
        let tiles_e = to_text(format!("Tiles: {} ", self.tiles).to_string());
        let leak_e = to_text(format!("Leak: {} ", self.leak).to_string());
        let score_e = to_text(format!("Score: {}", self.score).to_string());
        let score = crate::Element::from(iced::widget::Row::with_children(vec![
            tiles_e, leak_e, score_e,
        ]));
        let content = iced::widget::Column::with_children(vec![matrix, score]);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}

fn to_gui<'a>(y: usize, x: usize, s: CellState) -> Element<'a, Message> {
    let content = match s {
        CellState::Unused => {
            let button_text = "Heat".to_string();
            let button_content = to_text(button_text);
            let b1 = button(button_content).on_press(Message::Heat(y, x));
            let button_text = "Insulate".to_string();
            let button_content = to_text(button_text);
            let b2 = button(button_content).on_press(Message::Insulate(y, x));
            crate::Element::from(iced::widget::column!(b1, b2))
        }
        CellState::Insulation => to_text("Insulation".to_string()),
        CellState::Hot => to_text("Hot".to_string()),
    };
    crate::Element::from(container(content).width(100).height(100))
}

fn to_text<'a>(s: String) -> Element<'a, Message> {
    return crate::Element::from(text(s).size(20));
}

fn neighbors(x0: usize, y0: usize, m: &Vec<Vec<CellState>>) -> Vec<Option<CellState>> {
    let x: i32 = x0.try_into().unwrap();
    let y: i32 = y0.try_into().unwrap();
    let hard_neighbors = if ((y % 2) == 0) {
        [(x - 1, y + 1), (x + 1, y + 1)]
    } else {
        [(x - 1, y + 1), (x - 1, y - 1)]
    };
    let mut neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)].to_vec();
    neighbors.append(&mut hard_neighbors.to_vec());
    let ret = neighbors
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
                Some(&a) => Some(a),
            },
            None => None,
        })
        .collect();
    return ret;
}

fn leak_delta(x0: usize, y0: usize, m: &Vec<Vec<CellState>>) -> i32 {
    let n = neighbors(x0, y0, &m);
    let ret = n
        .iter()
        .map(|i| match i {
            Some(CellState::Hot) => -2,
            Some(CellState::Insulation) => 1,
            Some(CellState::Unused) => 2,
            None => 2,
        })
        .sum();
    ret
}

fn leak_delta_ins(x0: usize, y0: usize, m: &Vec<Vec<CellState>>) -> i32 {
    let n = neighbors(x0, y0, &m);
    let ret = n
        .iter()
        .map(|i| match i {
            Some(CellState::Hot) => -1,
            _ => 0,
        })
        .sum();
    ret
}
