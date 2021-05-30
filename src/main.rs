mod adapters_info;

use crate::adapters_info::AdaptersInfo;
use iced::{
    Application,
    Clipboard,
    Column,
    Command,
    Container,
    Element,
    Length,
    Row,
    Scrollable,
    Settings,
    Space,
    Text,
    TextInput,
};
use iced_aw::{
    graphics::icons::ICON_FONT,
    TabLabel,
};
use std::{
    ffi::OsString,
    time::Instant,
};
use winreg::{
    enums::{
        HKEY_LOCAL_MACHINE,
        KEY_ALL_ACCESS,
        KEY_ENUMERATE_SUB_KEYS,
    },
    RegKey,
};

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(usize),

    AdaptersInfo(crate::adapters_info::Message),

    Nop,
}

pub struct App {
    active_tab: usize,

    adapters_info: AdaptersInfo,

    scroll_state: iced::scrollable::State,

    registry_adapters: std::io::Result<Vec<std::io::Result<RegistryAdapter>>>,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let adapters_info = AdaptersInfo::new();

        let start = Instant::now();
        let registry_adapters = get_registry_adapters();
        println!("Got registry adapters in {:?}", start.elapsed());

        (
            App {
                active_tab: 0,

                adapters_info,

                scroll_state: iced::scrollable::State::new(),

                registry_adapters,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Hekk")
    }

    fn update(&mut self, message: Message, clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::TabSelected(new_active_tab) => {
                self.active_tab = new_active_tab;
                Command::none()
            }
            Message::AdaptersInfo(msg) => self
                .adapters_info
                .update(msg, clipboard)
                .map(Message::AdaptersInfo),
            Message::Nop => Command::none(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let title = Text::new("Mac Spoof").size(36);
        let mut column = Column::new().spacing(10).push(title);

        match self.registry_adapters.as_ref() {
            Ok(registry_adapters) => {
                for (i, registry_adapter) in registry_adapters.iter().enumerate() {
                    let info: Element<_> = match registry_adapter {
                        Ok(registry_adapter) => Column::new()
                            .push(
                                Text::new(format!(
                                    "Name: {}",
                                    registry_adapter
                                        .get_name()
                                        .unwrap_or_else(|e| e.to_string())
                                ))
                                .size(15),
                            )
                            .push(
                                Text::new(format!(
                                    "Description: {}",
                                    registry_adapter
                                        .get_description()
                                        .unwrap_or_else(|e| e.to_string())
                                ))
                                .size(15),
                            )
                            .push(
                                Text::new(format!(
                                    "Hardware Address: {}",
                                    registry_adapter
                                        .get_hardware_address()
                                        .map(|res| res
                                            .map(|s| s.to_string_lossy().into_owned())
                                            .unwrap_or_else(|| "Not Set".to_string()))
                                        .unwrap_or_else(|e| e.to_string())
                                ))
                                .size(15),
                            )
                            .into(),
                        Err(e) => Text::new(format!("Failed to get info: {}", e))
                            .size(15)
                            .into(),
                    };

                    let row = Row::new()
                        .push(Space::new(Length::Units(20), Length::Shrink))
                        .push(
                            Column::new()
                                .push(Text::new(format!("Adapter {}", i)))
                                .push(
                                    Row::new()
                                        .push(Space::new(Length::Units(20), Length::Shrink))
                                        .push(info),
                                ),
                        );

                    column = column.push(row);
                }
            }
            Err(e) => {
                column = column.push(Text::new(format!("Failed to get registry adapters: {}", e)));
            }
        }

        iced_aw::Tabs::new(self.active_tab, Message::TabSelected)
            .push(
                TabLabel::Text("Adapter Info".to_string()),
                self.adapters_info.view().map(Message::AdaptersInfo),
            )
            .push(
                TabLabel::Text("Spoof MAC".to_string()),
                Container::new(
                    Scrollable::new(&mut self.scroll_state)
                        .push(Container::new(column).padding(20))
                        .width(Length::Fill),
                )
                .style(GreyStyle)
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .tab_bar_style(GreyStyle)
            .icon_font(ICON_FONT)
            .width(Length::Fill)
            .height(Length::Fill)
            .tab_bar_position(iced_aw::TabBarPosition::Top)
            .into()
    }
}

#[derive(Debug)]
pub struct RegistryAdapter {
    key: RegKey,
}

impl RegistryAdapter {
    pub const HW_ADDRESS_KEY: &'static str = "NetworkAddress";

    pub fn from_key(key: RegKey) -> Self {
        RegistryAdapter { key }
    }

    pub fn get_description(&self) -> std::io::Result<String> {
        self.key.get_value("DriverDesc")
    }

    pub fn get_name(&self) -> std::io::Result<String> {
        self.key.get_value("NetCfgInstanceId")
    }

    /// Returns `None` if the hardware address does not exist
    pub fn get_hardware_address(&self) -> std::io::Result<Option<OsString>> {
        match self.key.get_value(Self::HW_ADDRESS_KEY) {
            Ok(addr) => Ok(Some(addr)),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn set_hardware_address(&self, hw_addr: &str) -> std::io::Result<()> {
        self.key.set_value(Self::HW_ADDRESS_KEY, &hw_addr)
    }

    pub fn reset_hardware_address(&self) -> std::io::Result<()> {
        self.key.delete_value(Self::HW_ADDRESS_KEY)
    }
}

/// This will try to get a read-only view of the adpater list, but writable adpaters.
/// You need admin access for this to work properly.
pub fn get_registry_adapters() -> std::io::Result<Vec<std::io::Result<RegistryAdapter>>> {
    pub const REGISTRY_ADAPTER_KEY_STR: &str =
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4D36E972-E325-11CE-BFC1-08002bE10318}";

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let main_key = hklm.open_subkey_with_flags(REGISTRY_ADAPTER_KEY_STR, KEY_ENUMERATE_SUB_KEYS)?;
    let iter = main_key.enum_keys();
    let keys = iter
        .filter_map(|key| match key {
            Ok(key) => {
                // According to windows docs, adapter's registry key names are always 4 bytes in the form "xxxx".
                if key.len() != 4 {
                    None
                } else {
                    Some(Ok(key))
                }
            }
            Err(e) => Some(Err(e)),
        })
        .map(|key_str| {
            main_key
                .open_subkey_with_flags(&key_str?, KEY_ALL_ACCESS)
                .map(RegistryAdapter::from_key)
        })
        .collect();
    Ok(keys)
}

fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size = (640, 480);
    App::run(settings)
}

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
