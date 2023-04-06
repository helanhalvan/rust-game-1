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

#[derive(Debug, Clone)]
enum CellState {
    Button,
    Text(String),
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ButtonPress(usize, usize),
}

impl Application for GameState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            GameState {
                matrix: vec![vec![CellState::Button; 10]; 5],
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Game")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ButtonPress(x, y) => {
                self.matrix[x][y] = CellState::Text("9,9".to_string());
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
        CellState::Button => {
            let button_text = format!("{},{}", y, x);
            let button_content = to_text(button_text);
            crate::Element::from(button(button_content).on_press(Message::ButtonPress(y, x)))
        }
        CellState::Text(t) => to_text(t),
    };
    crate::Element::from(container(content).width(100).height(100))
}

fn to_text(s: String) -> Element<'static, Message> {
    crate::Element::from(text(s).size(50))
}
