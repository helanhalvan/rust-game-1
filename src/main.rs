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

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::{dbg, env, fs, vec};

pub(crate) fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let _ = AppState::run(Settings {
            ..Settings::default()
        });
    } else if args[1] == "test" {
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
    get_img_done: RefCell<Option<Receiver<ImgDoneEvent>>>,
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
    ImgDone(ImgDoneEvent),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ImgDoneEvent {
    path: PathBuf,
    data: celldata::CellState,
}

fn read_reply_loop(
    mut done: HashSet<ImgDoneEvent>,
    rx: Receiver<celldata::CellState>,
    tx: Sender<ImgDoneEvent>,
) {
    loop {
        let data = rx.recv().unwrap();
        let path = make_imgs::make_image(data);
        let msg = ImgDoneEvent { path, data };
        if done.insert(msg.clone()) {
            tx.send(msg).unwrap();
        }
    }
}

impl Application for AppState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = css::Theme;
    type Flags = ();

    //if self was passed as mutable, this would be so much cleaner, no need for using a ref_cell sneaking in mutability
    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let a = iced_native::subscription::events().map(Message::NativeEvent);
        let b = iced::subscription::unfold(
            "img_done",
            self.queues.get_img_done.take(),
            move |mut receiver0| async move {
                let receiver = receiver0.as_mut().unwrap();
                let first = receiver.recv().unwrap();
                (Message::ImgDone(first), receiver0)
            },
        );
        subscription::Subscription::batch(vec![a, b])
    }

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let (s1, r1) = mpsc::channel();
        let (s2, r2) = mpsc::channel();
        std::thread::spawn(move || read_reply_loop(HashSet::new(), r1, s2));
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
                get_img_done: RefCell::new(Some(r2)),
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
            Message::ImgDone(i) => {
                if i.path.exists() && i.path.is_file() {
                    let data = fs::read(i.path).unwrap();
                    // This appears slower than keeping just filepaths in the handles,
                    // however with file_paths I got random images not rendering
                    // if there was a way to re-use the in-memory images for multiple
                    // renders that might speed things up
                    let handle = iced_native::image::Handle::from_memory(data);
                    self.game_state.img_buffer.insert(i.data, handle);
                } else {
                    unreachable!("{:?}", i);
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let start = std::time::Instant::now();
        let view_matrix = hexgrid::view_port(
            &self.game_state.matrix,
            self.game_state.io_cache.top_left_hex,
            self.game_state.io_cache.view_cells_x,
            self.game_state.io_cache.view_cells_y,
        );
        let _matrix_build = start.elapsed().as_millis();
        let x = view_matrix
            .map(|(x_index, i)| {
                let padding: Element<'static, Message> =
                    crate::Element::from(if (x_index as i32) % 2 == 0 {
                        container("")
                            .width(self.game_state.io_cache.cell_y_size)
                            .height(self.game_state.io_cache.cell_x_size / 2.0)
                    } else {
                        container("").width(0).height(0)
                    });
                let mut data: Vec<Element<'static, Message>> = i
                    .map(
                        |(
                            hexgrid::XYCont {
                                x: x_index,
                                y: y_index,
                            },
                            i,
                        )| {
                            visualize_cell::to_gui(
                                hexgrid::XYCont {
                                    x: x_index,
                                    y: y_index,
                                },
                                i.clone(),
                                &self.game_state,
                                &self.queues.send_img_job,
                            )
                        },
                    )
                    .collect();
                data.insert(0, padding);
                crate::Element::from(iced::widget::Column::with_children(data))
            })
            .collect();
        let _transform = start.elapsed().as_millis();
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

        let ret = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into();
        let _total = start.elapsed().as_millis();

        println!(
            "size:{} transform:{} matrix_build:{} other:{} total:{}",
            self.game_state.io_cache.view_cells_x,
            _matrix_build,
            _transform - _matrix_build,
            _total - _transform,
            _total
        );
        // */
        ret
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
