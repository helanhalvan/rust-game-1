use iced::{
    application, color,
    widget::{button, container, text},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Theme;

#[derive(Debug, Clone, Copy, Default)]
pub enum Container {
    #[default]
    Default,
    Bordered,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Default => container::Appearance {
                background: color!(0x0, 0x88, 0x0).into(),
                ..Default::default()
            },
            Container::Bordered => container::Appearance {
                background: color!(0x0, 0x88, 0x88).into(),
                border_color: color!(0, 0, 0),
                border_width: 1.0,
                border_radius: 4.0,
                ..Default::default()
            },
        }
    }
}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: color!(0x0, 0x88, 0x0),
            text_color: color!(0xff, 0xff, 0xff),
        }
    }
}

impl text::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: Self::Style) -> text::Appearance {
        Default::default()
    }
}

impl button::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        button::Appearance {
            background: color!(0x88, 0x88, 0x00).into(),
            border_width: 1.0,
            border_radius: 4.0,
            border_color: color!(0xff, 0xff, 0xff),
            text_color: color!(0xff, 0xff, 0xff),
            ..Default::default()
        }
    }
}
