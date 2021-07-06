use crate::{
    registry_adapter::RegistryAdapter,
    style::{
        ForegroundGreenTextInputStyle,
        GreyStyle,
    },
    ComThread,
};
use anyhow::Context;
use iced::{
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
use log::{
    error,
    info,
};
use std::{
    sync::Arc,
    time::Instant,
};

#[derive(Debug, Clone)]
pub enum Message {
    Adapter(usize, AdapterMessage),

    Nop,
}

pub struct MacSpoof {
    registry_adapters: std::io::Result<Vec<std::io::Result<Adapter>>>,

    com_thread: ComThread,

    scroll_state: iced::scrollable::State,
}

impl MacSpoof {
    pub fn new(com_thread: ComThread) -> Self {
        let mut ret = MacSpoof {
            registry_adapters: Err(std::io::Error::from_raw_os_error(0)),
            com_thread,
            scroll_state: iced::scrollable::State::new(),
        };
        ret.refresh_adapters();
        ret
    }

    pub fn refresh_adapters(&mut self) {
        let start = Instant::now();
        self.registry_adapters = RegistryAdapter::get_all().map(|registry_adapters| {
            registry_adapters
                .into_iter()
                .map(|adapter| adapter.map(Adapter::new))
                .collect()
        });
        info!("Got registry adapters in {:?}", start.elapsed());
    }

    pub fn update(&mut self, message: Message, clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Nop => Command::none(),
            Message::Adapter(i, message) => {
                match self
                    .registry_adapters
                    .as_mut()
                    .map(|registry_adapters| registry_adapters.get_mut(i))
                {
                    Ok(Some(Ok(adapter))) => adapter
                        .update(message, &self.com_thread, clipboard)
                        .map(move |msg| Message::Adapter(i, msg)),
                    Ok(Some(Err(_e))) => {
                        error!("Cannot process Adapter Message for adapter {} as it is in the error state: {:#?}", i, message);
                        Command::none()
                    }
                    Ok(None) => {
                        error!("Cannot process Adapter Message for adapter {} as it does not exist: {:#?}", i, message);
                        Command::none()
                    }
                    Err(_e) => {
                        error!("`registry_adapters` is in error state. Cannot process Adapter Message for adapter {}: {:#?}", i, message);
                        Command::none()
                    }
                }
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let title = Text::new("Mac Spoof").size(36);
        let mut column = Column::new().spacing(10).push(title);

        match self.registry_adapters.as_mut() {
            Ok(registry_adapters) => {
                for (i, registry_adapter) in registry_adapters.iter_mut().enumerate() {
                    let info: Element<_> = match registry_adapter {
                        Ok(registry_adapter) => registry_adapter.view(),
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

                    column = column.push(
                        Element::<AdapterMessage>::from(row)
                            .map(move |msg| Message::Adapter(i, msg)),
                    );
                }
            }
            Err(e) => {
                column = column.push(Text::new(format!("Failed to get registry adapters: {}", e)));
            }
        }

        Container::new(
            Scrollable::new(&mut self.scroll_state)
                .push(Container::new(column).padding(20))
                .width(Length::Fill),
        )
        .style(GreyStyle)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug, Clone)]
pub enum AdapterMessage {
    UpdateHardwareAddressField(String),
    SetHardwareAddress,
    DoneResetting(Arc<anyhow::Result<()>>),

    Nop,
}

pub struct Adapter {
    registry_adapter: RegistryAdapter,

    hardware_address: String,
    harware_address_state: iced::text_input::State,

    is_resetting: bool,
}

impl Adapter {
    pub fn new(registry_adapter: RegistryAdapter) -> Self {
        let mut ret = Adapter {
            registry_adapter,

            hardware_address: String::new(),
            harware_address_state: iced::text_input::State::new(),

            is_resetting: false,
        };
        ret.refresh_mac_address();
        ret
    }

    pub fn refresh_mac_address(&mut self) {
        self.hardware_address = self
            .registry_adapter
            .get_hardware_address()
            .map(|res| {
                res.map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(String::new)
            })
            .unwrap_or_else(|e| e.to_string());
    }

    pub fn update(
        &mut self,
        message: AdapterMessage,
        com_thread: &ComThread,
        _clipboard: &mut Clipboard,
    ) -> Command<AdapterMessage> {
        match message {
            AdapterMessage::UpdateHardwareAddressField(hardware_address) => {
                self.hardware_address = hardware_address;
                Command::none()
            }
            AdapterMessage::SetHardwareAddress => {
                let hardware_address = if self.hardware_address.is_empty() {
                    None
                } else {
                    Some(self.hardware_address.as_str().trim())
                };

                match hardware_address
                    .as_deref()
                    .map(validate_hardware_address)
                    .unwrap_or(Ok(()))
                {
                    Ok(()) => {
                        if let Err(e) = self
                            .registry_adapter
                            .set_hardware_address(hardware_address)
                            .context("failed to set hardware address")
                        {
                            // TODO: Give user visual feedback. Modal?
                            error!("{:?}", e);

                            Command::none()
                        } else {
                            info!(
                                "Set Hardware Address to {}",
                                hardware_address.unwrap_or("not set")
                            );

                            match self.registry_adapter.get_name() {
                                Ok(name) => {
                                    self.is_resetting = true;

                                    let com_thread = com_thread.clone();
                                    Command::perform(
                                        async move { com_thread.reset_network_connection(name).await },
                                        move |result| {
                                            AdapterMessage::DoneResetting(Arc::new(result))
                                        },
                                    )
                                }
                                Err(e) => {
                                    error!("Failed to get adapter name: {}", e);
                                    Command::none()
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // TODO: Give user visual feedback
                        error!("Invalid MAC Address: {}", e);
                        Command::none()
                    }
                }
            }
            AdapterMessage::DoneResetting(result) => {
                match result.as_ref() {
                    Ok(()) => {}
                    Err(e) => {
                        // TODO: Give user visual feedback
                        error!("Failed to reset adapter: {:?}", e);
                    }
                }

                self.is_resetting = false;
                self.refresh_mac_address();

                Command::none()
            }
            AdapterMessage::Nop => Command::none(),
        }
    }

    pub fn view(&mut self) -> Element<AdapterMessage> {
        let hardware_address = Row::new()
            .push(Text::new("Hardware Address: ").size(15))
            .push(
                TextInput::new(
                    &mut self.harware_address_state,
                    "Not Set",
                    &self.hardware_address,
                    AdapterMessage::UpdateHardwareAddressField,
                )
                .on_submit(AdapterMessage::SetHardwareAddress)
                .style(ForegroundGreenTextInputStyle)
                .size(15)
                .padding(2),
            );

        let column = Column::new()
            .push(
                Text::new(format!(
                    "Name: {}",
                    self.registry_adapter
                        .get_name()
                        .unwrap_or_else(|e| e.to_string())
                ))
                .size(15),
            )
            .push(
                Text::new(format!(
                    "Description: {}",
                    self.registry_adapter
                        .get_description()
                        .unwrap_or_else(|e| e.to_string())
                ))
                .size(15),
            )
            .push(Text::new(format!("Is Resetting: {}", self.is_resetting)).size(15))
            .push(hardware_address);

        column.into()
    }
}

/// Validates a mac address so that it is in the form XX-XX-XX-XX-XX-XX where X is a capital alphanumeric between A and F.
fn validate_hardware_address(hardware_address: &str) -> anyhow::Result<()> {
    let mut iter = hardware_address.chars();

    for i in 0..6 {
        for _ in 0..2 {
            let c = iter.next().context("address too short")?;
            if !('A'..='F').contains(&c) && !('a'..='f').contains(&c) && !('0'..='9').contains(&c) {
                anyhow::bail!("invalid char '{}'", c);
            }
        }

        if i != 5 {
            let c = iter.next().context("missing `-`")?;
            if c != '-' {
                anyhow::bail!("expected '-', got '{}'", c);
            }
        }
    }

    Ok(())
}
