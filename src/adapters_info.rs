use crate::style::{
    ForegroundGreenButtonStyle,
    ForegroundGreyContainerStyle,
    GreyStyle,
    GreyStyleCopyTextHack,
};
use iced::{
    Align,
    Button,
    Clipboard,
    Column,
    Command,
    Container,
    Element,
    Length,
    Row,
    Scrollable,
    Space,
    Text,
    TextInput,
};
use iphlpapi::{
    IpAdapterInfo,
    IpAddrString,
};
use log::info;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,

    Nop,
}

pub struct AdaptersInfo {
    adapters_info: std::io::Result<Vec<AdapterState>>,

    scroll_state: iced::scrollable::State,
    button_state: iced::button::State,
}

impl AdaptersInfo {
    pub fn new() -> Self {
        let mut ret = AdaptersInfo {
            adapters_info: Err(std::io::Error::from_raw_os_error(0)),

            scroll_state: iced::scrollable::State::new(),
            button_state: iced::button::State::new(),
        };
        ret.refresh_adapters_info();
        ret
    }

    pub fn refresh_adapters_info(&mut self) {
        let start = Instant::now();
        let adapters_info = iphlpapi::get_adapters_info();
        info!("Got adapters info in {:?}", start.elapsed());

        self.adapters_info = adapters_info
            .map(|adapters_info| adapters_info.iter().map(AdapterState::new).collect());
    }

    pub fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Refresh => {
                self.refresh_adapters_info();
                Command::none()
            }
            Message::Nop => Command::none(),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let title = Text::new("Adapter Info").size(36);
        let mut column = Column::new().spacing(10).push(title);

        match self.adapters_info.as_mut() {
            Ok(adapters) => {
                for (i, adapter_state) in adapters.iter_mut().enumerate() {
                    column = column.push(
                        Row::new()
                            .push(Space::new(Length::Units(20), Length::Shrink))
                            .push(adapter_state.view(i)),
                    );
                }
            }
            Err(e) => {
                column = column.push(Text::new(format!("Failed to get adapters: {}", e)));
            }
        }

        Container::new(
            Column::new()
                .push(
                    Scrollable::new(&mut self.scroll_state)
                        .padding(20)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .push(column),
                )
                .push(
                    Container::new(
                        Button::new(&mut self.button_state, Text::new("Refresh"))
                            .style(ForegroundGreenButtonStyle)
                            .on_press(Message::Refresh),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .align_x(Align::Center)
                    .align_y(Align::Center)
                    .style(ForegroundGreyContainerStyle),
                ),
        )
        .style(GreyStyle)
        .into()
    }
}

impl Default for AdaptersInfo {
    fn default() -> Self {
        Self::new()
    }
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

fn ip_address_list_view(ip_address_state_vec: &mut [IpAddress]) -> iced::Element<Message> {
    let mut column = Column::new();

    for state in ip_address_state_vec.iter_mut() {
        column = column.push(
            TextInput::new(&mut state.ip_address_state, "", &state.ip_address, |_| {
                Message::Nop
            })
            .style(GreyStyleCopyTextHack)
            .size(15),
        );

        column = column.push(
            TextInput::new(&mut state.mask_state, "", &state.mask, |_| Message::Nop)
                .style(GreyStyleCopyTextHack)
                .size(15),
        );
    }

    Row::new()
        .push(Space::new(Length::Units(20), Length::Shrink))
        .push(column)
        .into()
}

#[derive(Clone)]
struct AdapterState {
    name: String,
    name_state: iced::text_input::State,

    description: String,
    description_state: iced::text_input::State,

    combo_index: String,
    combo_index_state: iced::text_input::State,

    hardware_address: String,
    hardware_address_state: iced::text_input::State,

    ip_address_list: Vec<IpAddress>,
    gateway_address_list: Vec<IpAddress>,
}

impl AdapterState {
    pub fn new(adapter: &IpAdapterInfo) -> Self {
        AdapterState {
            name: format!("Name: {}", adapter.get_name().to_string_lossy()),
            name_state: iced::text_input::State::new(),

            description: format!(
                "Description: {}",
                adapter.get_description().to_string_lossy()
            ),
            description_state: iced::text_input::State::new(),

            combo_index: format!("Combo Index: {}", adapter.get_combo_index()),
            combo_index_state: iced::text_input::State::new(),

            hardware_address: format!(
                "Hardware Address: {}",
                format_address_to_string(adapter.get_address())
            ),
            hardware_address_state: iced::text_input::State::new(),

            ip_address_list: adapter
                .get_ip_address_list()
                .iter()
                .map(IpAddress::new)
                .collect(),
            gateway_address_list: adapter
                .get_gateway_list()
                .iter()
                .map(IpAddress::new)
                .collect(),
        }
    }

    fn view(&mut self, i: usize) -> iced::Element<Message> {
        let info_list_view = Row::new()
            .push(Space::new(Length::Units(20), Length::Shrink))
            .push(
                Column::new()
                    .push(
                        TextInput::new(&mut self.name_state, "", &self.name, |_| Message::Nop)
                            .style(GreyStyleCopyTextHack)
                            .size(15),
                    )
                    .push(
                        TextInput::new(&mut self.description_state, "", &self.description, |_| {
                            Message::Nop
                        })
                        .style(GreyStyleCopyTextHack)
                        .size(15),
                    )
                    .push(
                        TextInput::new(&mut self.combo_index_state, "", &self.combo_index, |_| {
                            Message::Nop
                        })
                        .style(GreyStyleCopyTextHack)
                        .size(15),
                    )
                    .push(
                        TextInput::new(
                            &mut self.hardware_address_state,
                            "",
                            &self.hardware_address,
                            |_| Message::Nop,
                        )
                        .style(GreyStyleCopyTextHack)
                        .size(15),
                    )
                    .push(Text::new("IP Address List").size(15))
                    .push(ip_address_list_view(&mut self.ip_address_list))
                    .push(Text::new("Gateway List").size(15))
                    .push(ip_address_list_view(&mut self.gateway_address_list)),
            );

        Column::new()
            .push(Text::new(format!("Adapter {}", i)))
            .push(info_list_view)
            .into()
    }
}

#[derive(Clone)]
pub struct IpAddress {
    ip_address: String,
    ip_address_state: iced::text_input::State,

    mask: String,
    mask_state: iced::text_input::State,
}

impl IpAddress {
    pub fn new(ip_address: &IpAddrString) -> Self {
        IpAddress {
            ip_address: format!("IP Address: {}", ip_address.get_address().to_string_lossy()),
            ip_address_state: iced::text_input::State::new(),

            mask: format!("Mask: {}", ip_address.get_mask().to_string_lossy()),
            mask_state: iced::text_input::State::new(),
        }
    }
}
