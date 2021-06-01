pub struct GreyStyle;

impl iced::container::StyleSheet for GreyStyle {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            background: iced::Color::from_rgb8(0x2C, 0x2C, 0x2C).into(),
            text_color: iced::Color::WHITE.into(),
            ..iced::container::Style::default()
        }
    }
}

impl iced_aw::tabs::StyleSheet for GreyStyle {
    fn active(&self, is_selected: bool) -> iced_aw::tabs::Style {
        let tab_label_background = if is_selected {
            iced::Color::from_rgb8(0x0A, 0x5D, 0x00).into()
        } else {
            iced::Color::from_rgb8(0x48, 0x77, 0x48).into()
        };

        let text_color = if is_selected {
            iced::Color::WHITE
        } else {
            iced::Color::BLACK
        };

        iced_aw::tabs::Style {
            background: None,
            border_color: None,
            border_width: 0.0,
            tab_label_background,
            tab_label_border_color: iced::Color::TRANSPARENT,
            tab_label_border_width: 0.0,
            icon_color: text_color,
            text_color,
        }
    }

    fn hovered(&self, is_selected: bool) -> iced_aw::tabs::Style {
        let tab_label_background = iced::Color::from_rgb8(0x06, 0x3B, 0x00).into();
        let text_color = iced::Color::WHITE;

        iced_aw::tabs::Style {
            tab_label_background,
            icon_color: text_color,
            text_color,
            ..self.active(is_selected)
        }
    }
}

pub struct GreyStyleCopyTextHack;

impl iced::widget::text_input::StyleSheet for GreyStyleCopyTextHack {
    fn active(&self) -> iced::widget::text_input::Style {
        iced::widget::text_input::Style {
            background: iced::Color::TRANSPARENT.into(),
            border_radius: 0.0,
            border_width: 0.0,
            ..Default::default()
        }
    }

    fn focused(&self) -> iced::widget::text_input::Style {
        iced::widget::text_input::Style {
            background: iced::Color::TRANSPARENT.into(),
            border_radius: 0.0,
            border_width: 0.0,
            ..Default::default()
        }
    }

    fn placeholder_color(&self) -> iced::Color {
        iced::Color::WHITE
    }

    fn value_color(&self) -> iced::Color {
        iced::Color::WHITE
    }

    fn selection_color(&self) -> iced::Color {
        iced::Color::from_rgb8(0x0A, 0x5D, 0x00)
    }
}

pub struct ForegroundGreyContainerStyle;

impl iced::container::StyleSheet for ForegroundGreyContainerStyle {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            background: iced::Color::from_rgb8(0x3F, 0x3F, 0x3F).into(),
            text_color: iced::Color::WHITE.into(),
            ..iced::container::Style::default()
        }
    }
}

pub struct ForegroundGreenButtonStyle;

impl iced::button::StyleSheet for ForegroundGreenButtonStyle {
    fn active(&self) -> iced::button::Style {
        iced::button::Style {
            background: Some(iced::Color::from_rgb8(0x0A, 0x5D, 0x00).into()),
            border_radius: 3.0,
            ..Default::default()
        }
    }

    fn hovered(&self) -> iced::button::Style {
        self.active()
    }

    fn pressed(&self) -> iced::button::Style {
        iced::button::Style {
            background: Some(iced::Color::from_rgb8(0x06, 0x3B, 0x00).into()),
            ..self.active()
        }
    }

    // pub fn disabled(&self) -> Style { ... }
}

pub struct ForegroundGreenTextInputStyle;

impl iced::widget::text_input::StyleSheet for ForegroundGreenTextInputStyle {
    fn active(&self) -> iced::widget::text_input::Style {
        iced::widget::text_input::Style {
            background: iced::Color::from_rgb8(0x06, 0x3B, 0x00).into(),
            border_radius: 5.0,
            ..Default::default()
        }
    }

    fn focused(&self) -> iced::widget::text_input::Style {
        self.active()
    }

    fn placeholder_color(&self) -> iced::Color {
        iced::Color::WHITE
    }

    fn value_color(&self) -> iced::Color {
        iced::Color::WHITE
    }

    fn selection_color(&self) -> iced::Color {
        iced::Color::from_rgb8(0x0A, 0x5D, 0x00)
    }
}
