pub(crate) mod actionmachine;
pub(crate) mod building;
pub(crate) mod celldata;
pub(crate) mod css;
pub(crate) mod hexgrid;
pub(crate) mod logistics_plane;
pub(crate) mod make_imgs;
pub(crate) mod make_world;
pub(crate) mod menu;
pub(crate) mod resource;
pub(crate) mod visualize_cell;

use iced::executor;
use iced::widget::{button, container};
use iced::{Application, Command, Length, Settings};
use iced_native::{row, subscription};
use widget::Element;

mod widget {
    use crate::css::Theme;
    //use iced::Theme;

    pub(crate) type Renderer = iced::Renderer<Theme>;
    pub(crate) type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
    //pub(crate) type Container<'a, Message> = iced::widget::Container<'a, Message, Renderer>;
    //pub(crate) type Button<'a, Message> = iced::widget::Button<'a, Message, Renderer>;
}

use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use std::{env, vec};

pub(crate) fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let _ = AppState::run(Settings {
            antialiasing: true,
            ..Settings::default()
        });
    } else if args[1] == "test" {
        make_world::test();
        dbg!(args);
    }
}

#[derive(Clone)]
pub(crate) struct GameState {
    matrix: hexgrid::Board,
    logistics_plane: logistics_plane::LogisticsPlane,
    resources: GameResources,
    action_machine: actionmachine::ActionMachine,
    img_buffer: visualize_cell::ImgBuffer,
    io_cache: IOCache,
}

pub(crate) struct AppState {
    game_state: GameState,
    queues: Queues,
}

#[derive(Debug)]
pub(crate) struct Queues {
    send_img_job: Sender<celldata::CellState>,
    get_img_done: Receiver<ImgDoneEvent>,
}

#[derive(Debug, Clone)]
pub(crate) struct IOCache {
    top_left_pos: iced_native::Point,
    latest_cursor: iced_native::Point,
    is_mousedown: bool,
    top_left_hex: hexgrid::XYCont<i32>,
    view_cells_x: i32,
    view_cells_y: i32,
    cell_x_size: f32,
    cell_y_size: f32,
    width_px: i32,
    height_px: i32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GameResources {
    tiles: i32,
    leak: i32,
    heat_efficency: f64,
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Build(celldata::CellStateVariant, hexgrid::Pos),
    EndTurn,
    Zoom(bool),
    NativeEvent(iced_native::Event),
    Tick(iced::time::Instant),
}

#[derive(Debug, Clone)]
pub(crate) struct ImgDoneEvent {
    path: String,
    data: celldata::CellState,
}

fn read_reply_loop(rx: Receiver<celldata::CellState>, tx: Sender<ImgDoneEvent>) {
    loop {
        let data = rx.recv().unwrap();
        let path = make_imgs::make_image(data);
        tx.send(ImgDoneEvent { path, data }).unwrap();
    }
}

impl Application for AppState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = css::Theme;
    type Flags = ();

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let a = iced_native::subscription::events().map(Message::NativeEvent);
        let b = iced::time::every(iced::time::Duration::from_millis(100)).map(Message::Tick);
        subscription::Subscription::batch(vec![a, b])
    }

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (s1, r1) = mpsc::channel();
        let (s2, r2) = mpsc::channel();
        std::thread::spawn(move || read_reply_loop(r1, s2));
        let start_x: i32 = 0;
        let start_y: i32 = 0;
        let start_view_cells_x = 7;
        let start_view_cells_y = 5;
        let start_cell_x_size = 100.0;
        let start_cell_y_size = 125.0;
        let width_px = 1000;
        let height_px = 1000;
        let m1 = make_world::new();
        let mut g = GameState {
            matrix: m1,
            logistics_plane: logistics_plane::new_plane(),
            resources: GameResources {
                tiles: 0,
                leak: 1,
                heat_efficency: 0.0,
            },
            action_machine: actionmachine::new(),
            img_buffer: visualize_cell::new_img_buffer(),
            io_cache: IOCache {
                top_left_pos: iced::Point {
                    x: (start_x as f32 - (start_view_cells_x / 2) as f32) * start_cell_x_size,
                    y: (start_y as f32 - (start_view_cells_y / 2) as f32) * start_cell_y_size,
                },
                latest_cursor: iced::Point { x: 0.0, y: 0.0 },
                is_mousedown: false,
                top_left_hex: hexgrid::XYCont {
                    x: start_x as i32 - (start_view_cells_x / 2),
                    y: start_y as i32 - (start_view_cells_y / 2),
                },
                view_cells_x: start_view_cells_x,
                view_cells_y: start_view_cells_y,
                cell_x_size: start_cell_x_size,
                cell_y_size: start_cell_y_size,
                width_px: width_px,
                height_px: height_px,
            },
        };
        let p = hexgrid::Pos {
            x: start_x,
            y: start_y,
        };
        let cv = celldata::CellStateVariant::Hub;
        g = building::do_build(actionmachine::Other::CellStateVariant(cv), p, g);
        let mut start_hub = hexgrid::get(p, &mut g.matrix);
        start_hub = resource::add(resource::ResourceType::Wood, start_hub, 10).unwrap();
        hexgrid::set(p, start_hub, &mut g.matrix);
        let a = AppState {
            game_state: g,
            queues: Queues {
                send_img_job: s1,
                get_img_done: r2,
            },
        };
        (a, Command::none())
    }

    fn title(&self) -> String {
        String::from("Game")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Build(t, pos) => {
                self.game_state = building::build(t, pos, self.game_state.clone())
            }
            Message::EndTurn => {
                self.game_state = actionmachine::run(self.game_state.clone());
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::CursorMoved {
                position,
            })) => {
                if self.game_state.io_cache.is_mousedown == true {
                    let old_p = self.game_state.io_cache.latest_cursor;
                    let delta = old_p - position;
                    self.game_state.io_cache.top_left_pos =
                        self.game_state.io_cache.top_left_pos + delta;
                    re_calc_cells_in_view(&mut self.game_state)
                }
                self.game_state.io_cache.latest_cursor = position;
            }
            Message::Zoom(is_out) => {
                if is_out {
                    self.game_state.io_cache.cell_x_size =
                        self.game_state.io_cache.cell_x_size / visualize_cell::ZOOM_FACTOR;
                    self.game_state.io_cache.cell_y_size =
                        self.game_state.io_cache.cell_y_size / visualize_cell::ZOOM_FACTOR;
                } else {
                    self.game_state.io_cache.cell_x_size =
                        self.game_state.io_cache.cell_x_size * visualize_cell::ZOOM_FACTOR;
                    self.game_state.io_cache.cell_y_size =
                        self.game_state.io_cache.cell_y_size * visualize_cell::ZOOM_FACTOR;
                }
                re_calc_cells_in_view(&mut self.game_state)
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                iced::mouse::Button::Left,
            ))) => {
                self.game_state.io_cache.is_mousedown = true;
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            ))) => {
                self.game_state.io_cache.is_mousedown = false;
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::CursorLeft)) => {
                self.game_state.io_cache.is_mousedown = false;
            }
            Message::NativeEvent(iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
                delta: iced_native::mouse::ScrollDelta::Lines { y, .. },
            })) => {
                let d = y.abs() * visualize_cell::ZOOM_FACTOR;
                if y < 0.0 {
                    self.game_state.io_cache.cell_x_size = self.game_state.io_cache.cell_x_size / d;
                    self.game_state.io_cache.cell_y_size = self.game_state.io_cache.cell_y_size / d;
                } else {
                    self.game_state.io_cache.cell_x_size = self.game_state.io_cache.cell_x_size * d;
                    self.game_state.io_cache.cell_y_size = self.game_state.io_cache.cell_y_size * d;
                }
                re_calc_cells_in_view(&mut self.game_state)
            }
            Message::NativeEvent(iced::Event::Window(iced::window::Event::Resized {
                width,
                height,
            })) => {
                self.game_state.io_cache.width_px = width as i32;
                self.game_state.io_cache.height_px = height as i32;
                re_calc_cells_in_view(&mut self.game_state)
            }
            Message::NativeEvent(_) => {}
            Message::Tick(_) => {
                while let Ok(new) = self
                    .queues
                    .get_img_done
                    .recv_timeout(Duration::from_millis(0))
                {
                    self.game_state
                        .img_buffer
                        .insert(new.data, iced_native::image::Handle::from_path(new.path));
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let view_matrix = hexgrid::view_port(
            &self.game_state.matrix,
            self.game_state.io_cache.top_left_hex,
            self.game_state.io_cache.view_cells_x - 1,
            self.game_state.io_cache.view_cells_y - 1,
        );
        let hexgrid::XYCont {
            x: base_x,
            y: base_y,
        } = self.game_state.io_cache.top_left_hex;
        let x = view_matrix
            .iter()
            .enumerate()
            .map(|(x_index, i)| {
                let padding: Element<'static, Message> =
                    crate::Element::from(if (base_x + x_index as i32) % 2 == 0 {
                        container("")
                            .width(self.game_state.io_cache.cell_y_size)
                            .height(self.game_state.io_cache.cell_x_size / 2.0)
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
                            &self.game_state,
                            &self.queues.send_img_job,
                        )
                    })
                    .collect();
                data.insert(0, padding);
                crate::Element::from(iced::widget::Column::with_children(data))
            })
            .collect();
        let matrix = crate::Element::from(iced::widget::Row::with_children(x));
        let resources = crate::Element::from(visualize_cell::to_text(
            format!("{:?}", self.game_state.resources).to_string(),
        ));
        let end_turn_content = visualize_cell::to_text("End Turn".to_string());
        let zoom_out_content = visualize_cell::to_text("Zoom Out".to_string());
        let zoom_in_content = visualize_cell::to_text("Zoom In".to_string());

        let buttom_buttons = crate::Element::from(row![
            button(end_turn_content).on_press(Message::EndTurn),
            button(zoom_out_content).on_press(Message::Zoom(true)),
            button(zoom_in_content).on_press(Message::Zoom(false)),
        ]);
        let ui_misc = crate::Element::from(row![visualize_cell::to_text(
            format!(
                "{:?}",
                self.game_state.io_cache.view_cells_x * self.game_state.io_cache.view_cells_y
            )
            .to_string()
        ),]);
        let content =
            iced::widget::Column::with_children(vec![matrix, resources, buttom_buttons, ui_misc]);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }
}

fn re_calc_cells_in_view(g: &mut GameState) {
    (*g).io_cache.top_left_hex = approx((*g).io_cache.top_left_pos, g);
    (*g).io_cache.view_cells_x = (*g).io_cache.width_px as i32 / (*g).io_cache.cell_x_size as i32;
    (*g).io_cache.view_cells_y = (*g).io_cache.height_px as i32 / (*g).io_cache.cell_y_size as i32;
    hexgrid::touch_all_chunks(
        &mut g.matrix,
        g.io_cache.top_left_hex,
        g.io_cache.view_cells_x - 1,
        g.io_cache.view_cells_y - 1,
    );
}

fn approx(iced::Point { x, y }: iced_native::Point, g: &GameState) -> hexgrid::XYCont<i32> {
    hexgrid::XYCont {
        x: (x / g.io_cache.cell_x_size) as i32,
        y: (y / g.io_cache.cell_y_size) as i32,
    }
}
