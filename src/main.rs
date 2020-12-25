mod geometry_conversion;
mod number_input;

use geometry_conversion::GeometryConversion;
use glam::{vec2, Vec2};
use iced::{
    canvas::{self, path, Frame, LineCap, LineJoin, Path, Program, Stroke},
    executor, slider, time, Application, Canvas, Checkbox, Color, Column, Command,
    HorizontalAlignment, Length, Row, Settings, Slider, Subscription, Text, VerticalAlignment,
};
use std::{
    cell::Cell,
    f32::consts::TAU,
    time::{Duration, Instant},
};

const FRAMERATE: u64 = 60;
const PATH_TABULATION_SIZE: usize = 500;

pub fn main() -> iced::Result {
    Model::run(Settings {
        antialiasing: true,
        ..Default::default()
    })
}

fn calc_points(oa: f32, ab: f32, am_per_ab: f32, progress: f32) -> (Vec2, Vec2, Vec2) {
    let t = progress * TAU;
    let ob = match oa.partial_cmp(&ab).unwrap() {
        std::cmp::Ordering::Less => oa * t.cos() + ab,
        std::cmp::Ordering::Equal => (oa + ab) * t.cos(),
        std::cmp::Ordering::Greater => oa + ab * t.cos(),
    };
    let b: Vec2 = vec2(ob, 0.0);
    let a_x = (ob * ob + oa * oa - ab * ab) / (2.0 * ob);
    let a_y = (oa * oa - a_x * a_x).sqrt().copysign(progress - 0.5);
    let a: Vec2 = vec2(a_x, a_y);
    let m: Vec2 = if ab != 0.0 {
        a + am_per_ab * (b - a)
    } else {
        a
    };

    (a, b, m)
}
fn tabulate_path(oa: f32, ab: f32, am_per_ab: f32, path: &mut [iced::Point]) {
    let n = path.len();
    for (i, x) in path.iter_mut().enumerate() {
        *x = calc_points(oa, ab, am_per_ab, i as f32 / n as f32)
            .2
            .point();
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    OaChange(f32),
    AbChange(f32),
    AmPerAbChange(f32),
    PeriodChange(Duration),
    Progress(f32),
    AutorunToggle(bool),
    TracePathToggle(bool),
    Tick(Instant),
}

struct Model {
    scematic: Schematic,
    oa_state: number_input::State,
    ab_state: number_input::State,
    am_per_ab_state: slider::State,
    period_state: number_input::State,
    progress: slider::State,
    autorun: bool,
    last_frame: Instant,
    period: Duration,
}

impl Application for Model {
    type Message = Option<Message>;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Model, Command<Option<Message>>) {
        let scematic = Schematic::new();
        let oa_state = number_input::State::new(scematic.oa);
        let ab_state = number_input::State::new(scematic.ab);
        let am_per_ab_state = slider::State::new();

        (
            Model {
                scematic,
                oa_state,
                ab_state,
                am_per_ab_state,
                period_state: number_input::State::new(3),
                progress: slider::State::new(),
                autorun: true,
                last_frame: Instant::now(),
                period: Duration::from_secs(3),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Computer Graphics Lab 6".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Option<Message>> {
        match message {
            Some(Message::OaChange(oa)) => {
                self.scematic.set_oa(oa);
            }
            Some(Message::AbChange(ab)) => {
                self.scematic.set_ab(ab);
            }
            Some(Message::AmPerAbChange(am)) => {
                self.scematic.set_am(am);
            }
            Some(Message::PeriodChange(period)) => self.period = period,
            Some(Message::Progress(progress)) => self.scematic.progress = progress,
            Some(Message::AutorunToggle(autorun)) => {
                self.last_frame = Instant::now();
                self.autorun = autorun;
            }
            Some(Message::TracePathToggle(trace_path)) => self.scematic.trace_path = trace_path,
            Some(Message::Tick(current_frame)) => {
                let period = self.period.as_secs_f32();
                if period != 0.0 {
                    self.scematic.progress = (self.scematic.progress
                        + (current_frame - self.last_frame).as_secs_f32() / period)
                        % 1.0;
                }
                self.last_frame = current_frame;
            }
            None => (),
        }
        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        let Schematic {
            progress,
            trace_path,
            am_per_ab,
            ..
        } = self.scematic;
        Column::new()
            .push(
                Row::new()
                    .push(
                        Canvas::new(&mut self.scematic)
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                    .push(
                        Column::new()
                            .push(Row::new().push(Text::new("OA: ")).push(number_input::new(
                                &mut self.oa_state,
                                "OA",
                                |oa| oa.map(Message::OaChange),
                            )))
                            .push(Row::new().push(Text::new("AB: ")).push(number_input::new(
                                &mut self.ab_state,
                                "AB",
                                |ab| ab.map(Message::AbChange),
                            )))
                            .push(Text::new("AM per AB: "))
                            .push(
                                Slider::new(
                                    &mut self.am_per_ab_state,
                                    0.0..=1.0,
                                    am_per_ab,
                                    |am_per_ab| Some(Message::AmPerAbChange(am_per_ab)),
                                )
                                .step(0.001),
                            )
                            .push(
                                Row::new()
                                    .push(Text::new("Period: "))
                                    .push(number_input::new(
                                        &mut self.period_state,
                                        "seconds",
                                        |x| {
                                            x.map(Duration::from_secs_f32)
                                                .map(Message::PeriodChange)
                                        },
                                    )),
                            )
                            .push(Checkbox::new(self.autorun, "Autorun", |autorun| {
                                Some(Message::AutorunToggle(autorun))
                            }))
                            .push(Checkbox::new(
                                trace_path,
                                "Trace M point's path",
                                |trace_path| Some(Message::TracePathToggle(trace_path)),
                            ))
                            .width(Length::Units(200))
                            .height(Length::Fill)
                            .spacing(20)
                            .padding(20),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .push(
                Slider::new(&mut self.progress, 0.0..=1.0, progress, |x| {
                    Some(Message::Progress(x))
                })
                .step(0.001),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        const FRAME: Duration = Duration::from_micros(1_000_000 / FRAMERATE);
        if self.autorun {
            time::every(FRAME).map(|i| Some(Message::Tick(i)))
        } else {
            Subscription::none()
        }
    }
}

#[derive(Debug, Clone)]
struct Schematic {
    oa: f32,
    ab: f32,
    am_per_ab: f32,
    progress: f32,
    m_path: [iced::Point; PATH_TABULATION_SIZE],
    trace_path: bool,
}

impl Schematic {
    fn new() -> Schematic {
        let mut new = Schematic {
            oa: 1.0,
            ab: 1.0,
            am_per_ab: 0.5,
            progress: 0.0,
            m_path: [iced::Point::new(0.0, 0.0); PATH_TABULATION_SIZE],
            trace_path: false,
        };
        new.tabulate_path();
        new
    }
    fn set_oa(&mut self, oa: f32) {
        self.oa = oa;
        self.tabulate_path();
    }
    fn set_ab(&mut self, ab: f32) {
        self.ab = ab;
        self.tabulate_path();
    }
    fn set_am(&mut self, am: f32) {
        self.am_per_ab = am;
        self.tabulate_path();
    }
    fn tabulate_path(&mut self) {
        tabulate_path(self.oa, self.ab, self.am_per_ab, &mut self.m_path);
    }
}

impl Program<Option<Message>> for Schematic {
    fn draw(
        &self,
        bounds: iced::Rectangle,
        _cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        let mut frame = Frame::new(bounds.size());

        const SCHEMATIC_STROKE: Stroke = Stroke {
            color: Color::BLACK,
            width: 1.5,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
        };

        let point_radius = Cell::new(3.0);
        let custom_scale = |frame: &mut Frame, scale: f32| {
            point_radius.set(point_radius.get() / scale);
            frame.scale(scale);
        };

        let make_point = |frame: &mut Frame, position: iced::Point| {
            frame.fill(&Path::circle(position, point_radius.get()), Color::BLACK);
        };

        fn make_text(frame: &mut Frame, content: String, position: impl GeometryConversion) {
            frame.fill_text(canvas::Text {
                content,
                position: position.point(),
                color: Color::BLACK,
                size: 20.0,
                font: iced::Font::Default,
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Top,
            })
        }

        fn make_line(
            frame: &mut Frame,
            from: impl GeometryConversion,
            to: impl GeometryConversion,
        ) {
            frame.stroke(&Path::line(from.point(), to.point()), SCHEMATIC_STROKE);
        }

        fn make_arrow(frame: &mut Frame, head: Vec2, tail: Vec2) {
            const HEAD_WIDTH: f32 = 0.025;
            const HEAD_HEIGHT: f32 = 0.05;
            let normal = (head - tail).normalize();
            let side_normal = normal.perp();
            let head_sides = (
                head - normal * HEAD_HEIGHT + side_normal * HEAD_WIDTH / 2.0,
                head - normal * HEAD_HEIGHT - side_normal * HEAD_WIDTH / 2.0,
            );

            make_line(frame, head, tail);
            make_line(frame, head, head_sides.0);
            make_line(frame, head, head_sides.1);
        }

        frame.translate(bounds.center().vector().vector());
        custom_scale(&mut frame, bounds.width.min(bounds.height) / 2.0);
        custom_scale(&mut frame, 0.95);
        // axes
        {
            make_text(&mut frame, "y".to_string(), vec2(0.03, -1.0));
            make_arrow(&mut frame, vec2(0.0, -1.0), vec2(0.0, 1.0));
            make_text(&mut frame, "x".to_string(), vec2(0.95, 0.0));
            make_arrow(&mut frame, vec2(1.0, 0.0), vec2(-1.0, 0.0));
        }
        custom_scale(&mut frame, 0.9 / (self.oa + self.ab));
        // hadle
        {
            let o: Vec2 = vec2(0.0, 0.0);
            let (a, b, m) = calc_points(self.oa, self.ab, self.am_per_ab, self.progress);

            make_point(&mut frame, o.point());
            make_text(&mut frame, "O".to_string(), o);
            make_point(&mut frame, a.point());
            make_text(&mut frame, "A".to_string(), a);
            make_line(&mut frame, o, a);
            make_point(&mut frame, b.point());
            make_text(&mut frame, "B".to_string(), b);
            make_line(&mut frame, a, b);
            make_point(&mut frame, m.point());
            make_text(&mut frame, "M".to_string(), m);
        }
        // path
        if self.trace_path {
            let mut builder = path::Builder::new();
            builder.move_to(iced::Point::new(0.0, 0.0));
            let n = (self.progress * PATH_TABULATION_SIZE as f32) as usize;
            for point in &self.m_path[..n] {
                builder.line_to(*point);
            }
            frame.stroke(&builder.build(), SCHEMATIC_STROKE);
        }

        vec![frame.into_geometry()]
    }
}
