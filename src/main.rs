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
use iphlpapi::IpAdapterInfoList;
use std::time::Instant;

// use clap::App;

//fn main() {
/*
let matches = App::new("Hekk")
    .author("adumbidiot")
    .about("A tool to hekk things")
    .subcommand(hekk::commands::adapter_info::cli())
    .get_matches();

match matches.subcommand() {
    ("adapter-info", Some(matches)) => hekk::commands::adapter_info::exec(&matches),
    (cmd, _) => {
        println!("Unknown command: {:#?}", cmd);
    }
}
*/
//}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(usize),

    Nop,
}

pub struct App {
    active_tab: usize,

    adapters_info_state: AdaptersInfoState,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let adapters_info_state = AdaptersInfoState::new();

        (
            App {
                active_tab: 0,
                adapters_info_state,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Hekk")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::TabSelected(new_active_tab) => {
                self.active_tab = dbg!(new_active_tab);

                Command::none()
            }
            Message::Nop => Command::none(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let adapters_info_view = adapters_info_view(&mut self.adapters_info_state);

        iced_aw::Tabs::new(self.active_tab, Message::TabSelected)
            .push(
                TabLabel::Text("Adapter Info".to_string()),
                adapters_info_view,
            )
            .tab_bar_style(GreyStyle)
            .icon_font(ICON_FONT)
            .width(Length::Fill)
            .height(Length::Fill)
            .tab_bar_position(iced_aw::TabBarPosition::Top)
            .into()
    }
}

pub struct AdaptersInfoState {
    adapters_info: Result<IpAdapterInfoList, std::io::Error>,

    scroll_state: iced::scrollable::State,

    adapter_info_state_vec: Vec<AdapterState>,
}

impl AdaptersInfoState {
    pub fn new() -> Self {
        let mut ret = AdaptersInfoState {
            adapters_info: Err(std::io::Error::from_raw_os_error(0)),
            scroll_state: iced::scrollable::State::new(),

            adapter_info_state_vec: Vec::new(),
        };
        ret.regenerate_adapters_info();
        ret
    }

    pub fn regenerate_adapters_info(&mut self) {
        let start = Instant::now();
        self.adapters_info = iphlpapi::get_adapters_info();
        println!("Got adapters info in {:?}", start.elapsed());

        self.adapter_info_state_vec.clear();
        self.adapter_info_state_vec.resize_with(
            self.adapters_info
                .as_ref()
                .map_or(0, |adapters_info| adapters_info.iter().count()),
            AdapterState::new,
        );

        if let Ok(adapters_info) = self.adapters_info.as_ref() {
            for (adapter, state) in adapters_info
                .iter()
                .zip(self.adapter_info_state_vec.iter_mut())
            {
                state.ip_address_state_vec.clear();
                state.ip_address_state_vec.resize_with(
                    adapter.get_ip_address_list().iter().count(),
                    IpAddressState::new,
                );

                state.gateway_address_state_vec.clear();
                state.gateway_address_state_vec.resize_with(
                    adapter.get_gateway_list().iter().count(),
                    IpAddressState::new,
                );
            }
        }
    }
}

impl Default for AdaptersInfoState {
    fn default() -> Self {
        Self::new()
    }
}

fn adapters_info_view<'a>(state: &'a mut AdaptersInfoState) -> Element<'a, Message> {
    let title = Text::new("Adapter Info").size(36);
    let mut column = Column::new().spacing(10).push(title);

    match state.adapters_info.as_ref() {
        Ok(adapters) => {
            for (i, (adapter, adapter_state)) in adapters
                .iter()
                .zip(state.adapter_info_state_vec.iter_mut())
                .enumerate()
            {
                let ip_address_list_view = {
                    let mut column = Column::new();

                    for (ip, state) in adapter
                        .get_ip_address_list()
                        .iter()
                        .zip(adapter_state.ip_address_state_vec.iter_mut())
                    {
                        column = column.push(
                            TextInput::new(
                                &mut state.ip_address_state,
                                "",
                                &format!("IP Address: {}", ip.get_address().to_string_lossy()),
                                |_| Message::Nop,
                            )
                            .style(GreyStyleCopyTextHack)
                            .size(15),
                        );

                        column = column.push(
                            TextInput::new(
                                &mut state.mask_state,
                                "",
                                &format!("Mask: {}", ip.get_mask().to_string_lossy()),
                                |_| Message::Nop,
                            )
                            .style(GreyStyleCopyTextHack)
                            .size(15),
                        );
                    }

                    Row::new()
                        .push(Space::new(Length::Units(20), Length::Shrink))
                        .push(column)
                };

                let gateway_list_view = {
                    let mut column = Column::new();
                    for (ip, state) in adapter
                        .get_gateway_list()
                        .iter()
                        .zip(adapter_state.gateway_address_state_vec.iter_mut())
                    {
                        column = column.push(
                            TextInput::new(
                                &mut state.ip_address_state,
                                "",
                                &format!("Gateway: {}", ip.get_address().to_string_lossy()),
                                |_| Message::Nop,
                            )
                            .style(GreyStyleCopyTextHack)
                            .size(15),
                        );

                        column = column.push(
                            TextInput::new(
                                &mut state.mask_state,
                                "",
                                &format!("Mask: {}", ip.get_mask().to_string_lossy()),
                                |_| Message::Nop,
                            )
                            .style(GreyStyleCopyTextHack)
                            .size(15),
                        );
                    }

                    Row::new()
                        .push(Space::new(Length::Units(20), Length::Shrink))
                        .push(column)
                };

                let info_list_view = Row::new()
                    .push(Space::new(Length::Units(20), Length::Shrink))
                    .push(
                        Column::new()
                            .push(
                                TextInput::new(
                                    &mut adapter_state.name_state,
                                    "",
                                    &format!("Name: {}", adapter.get_name().to_string_lossy()),
                                    |_| Message::Nop,
                                )
                                .style(GreyStyleCopyTextHack)
                                .size(15),
                            )
                            .push(
                                TextInput::new(
                                    &mut adapter_state.description_state,
                                    "",
                                    &format!(
                                        "Description: {}",
                                        adapter.get_description().to_string_lossy()
                                    ),
                                    |_| Message::Nop,
                                )
                                .style(GreyStyleCopyTextHack)
                                .size(15),
                            )
                            .push(
                                TextInput::new(
                                    &mut adapter_state.combo_index_state,
                                    "",
                                    &format!("Combo Index: {}", adapter.get_combo_index()),
                                    |_| Message::Nop,
                                )
                                .style(GreyStyleCopyTextHack)
                                .size(15),
                            )
                            .push(
                                TextInput::new(
                                    &mut adapter_state.hardware_address_state,
                                    "",
                                    &format!(
                                        "Hardware Address: {}",
                                        format_address_to_string(adapter.get_address())
                                    ),
                                    |_| Message::Nop,
                                )
                                .style(GreyStyleCopyTextHack)
                                .size(15),
                            )
                            .push(Text::new("IP Address List").size(15))
                            .push(ip_address_list_view)
                            .push(Text::new("Gateway List").size(15))
                            .push(gateway_list_view),
                    );

                let adapter_view = Row::new()
                    .push(Space::new(Length::Units(20), Length::Shrink))
                    .push(
                        Column::new()
                            .push(Text::new(format!("Adapter {}", i)))
                            .push(info_list_view),
                    );
                column = column.push(adapter_view);
            }
        }
        Err(e) => {
            column = column.push(Text::new(format!("Failed to get adapters: {}", e)));
        }
    }

    Container::new(
        Scrollable::new(&mut state.scroll_state)
            .width(Length::Fill)
            .push(
                Container::new(Column::new().push(column))
                    .padding(10)
                    .height(Length::Shrink)
                    .style(GreyStyle),
            ),
    )
    .height(Length::Fill)
    .style(GreyStyle)
    .into()
}

fn format_address_to_string(address: &[u8]) -> String {
    let mut ret = String::new();
    format_address(&mut ret, address).expect("failed to format hardware address");
    ret
}

fn format_address(mut f: impl std::fmt::Write, data: &[u8]) -> std::fmt::Result {
    for (i, b) in data.iter().enumerate() {
        if i == data.len() - 1 {
            write!(f, "{:02X}", b)?;
        } else {
            write!(f, "{:02X}-", b)?;
        }
    }
    writeln!(f)?;
    Ok(())
}

#[derive(Clone)]
struct AdapterState {
    name_state: iced::text_input::State,
    description_state: iced::text_input::State,
    combo_index_state: iced::text_input::State,
    hardware_address_state: iced::text_input::State,

    ip_address_state_vec: Vec<IpAddressState>,
    gateway_address_state_vec: Vec<IpAddressState>,
}

impl AdapterState {
    pub fn new() -> Self {
        AdapterState {
            name_state: iced::text_input::State::new(),
            description_state: iced::text_input::State::new(),
            combo_index_state: iced::text_input::State::new(),
            hardware_address_state: iced::text_input::State::new(),

            ip_address_state_vec: Vec::new(),
            gateway_address_state_vec: Vec::new(),
        }
    }
}

impl Default for AdapterState {
    fn default() -> Self {
        AdapterState::new()
    }
}

#[derive(Clone)]
pub struct IpAddressState {
    ip_address_state: iced::text_input::State,
    mask_state: iced::text_input::State,
}

impl IpAddressState {
    pub fn new() -> Self {
        IpAddressState {
            ip_address_state: iced::text_input::State::new(),
            mask_state: iced::text_input::State::new(),
        }
    }
}

impl Default for IpAddressState {
    fn default() -> Self {
        Self::new()
    }
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
            iced::Background::Color(iced::Color::WHITE)
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
