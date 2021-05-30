use crate::{
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
use iphlpapi::IpAdapterInfoList;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,

    Nop,
}

pub struct AdaptersInfo {
    adapters_info: std::io::Result<IpAdapterInfoList>,

    scroll_state: iced::scrollable::State,
    button_state: iced::button::State,

    adapter_info_state_vec: Vec<AdapterState>,
}

impl AdaptersInfo {
    pub fn new() -> Self {
        let mut ret = AdaptersInfo {
            adapters_info: Err(std::io::Error::from_raw_os_error(0)),

            scroll_state: iced::scrollable::State::new(),
            button_state: iced::button::State::new(),

            adapter_info_state_vec: Vec::new(),
        };
        ret.refresh_adapters_info();
        ret
    }

    pub fn refresh_adapters_info(&mut self) {
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

        match self.adapters_info.as_ref() {
            Ok(adapters) => {
                for (i, (adapter, adapter_state)) in adapters
                    .iter()
                    .zip(self.adapter_info_state_vec.iter_mut())
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
