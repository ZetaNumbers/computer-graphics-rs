use iced::{text_input, TextInput};
use std::{
    cell::RefCell,
    fmt::{write, Display},
    rc::Rc,
    str::FromStr,
};
use style::Theme;

mod style {
    use iced::text_input;

    #[derive(Debug, Clone, Copy)]
    pub enum Theme {
        Ok,
        Err,
    }

    impl From<Theme> for Box<dyn text_input::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Ok => Default::default(),
                Theme::Err => error::TextInput.into(),
            }
        }
    }

    mod error {
        use iced::{text_input, Color};

        const ERROR_BACKGROUND: Color = Color::from_rgb(1.0, 0.8, 0.8);

        pub struct TextInput;

        impl text_input::StyleSheet for TextInput {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: ERROR_BACKGROUND.into(),
                    border_radius: 5.0,
                    border_width: 1.0,
                    border_color: Color::from_rgb(0.7, 0.7, 0.7),
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..self.active()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color::from_rgb(0.7, 0.7, 0.7)
            }

            fn value_color(&self) -> Color {
                Color::from_rgb(0.3, 0.3, 0.3)
            }

            fn selection_color(&self) -> Color {
                Color::from_rgb(0.8, 0.8, 1.0)
            }
        }
    }
}

struct InnerState {
    theme: Theme,
    buffer: String,
}

pub struct State {
    state: text_input::State,
    inner: Rc<RefCell<InnerState>>,
}

impl State {
    pub fn new<T>(value: T) -> Self
    where
        T: Display,
    {
        let mut buffer = String::new();
        write(&mut buffer, format_args!("{}", value)).unwrap();
        Self {
            state: text_input::State::new(),
            inner: Rc::new(RefCell::new(InnerState {
                buffer,
                theme: style::Theme::Ok,
            })),
        }
    }
}

pub fn new<'a, T, F, Message>(
    state: &'a mut State,
    placeholder: &str,
    on_change: F,
) -> TextInput<'a, Message>
where
    T: Display + FromStr,
    F: 'static + Fn(Option<T>) -> Message,
    Message: Clone,
{
    let inner = state.inner.clone();
    TextInput::new(
        &mut state.state,
        placeholder,
        &state.inner.borrow().buffer,
        move |s| {
            let mut inner = inner.borrow_mut();
            inner.buffer = s;
            let x = T::from_str(&inner.buffer).ok();
            inner.theme = match x {
                Some(_) => Theme::Ok,
                None => Theme::Err,
            };
            on_change(x)
        },
    )
    .style(state.inner.borrow().theme)
}
