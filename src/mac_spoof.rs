use crate::{
    ForegroundGreenTextInputStyle,
    GreyStyle,
};
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
use netcon::{
    NetConProperties,
    NetConnection,
    NetConnectionManager,
};
use std::{
    ffi::OsString,
    time::Instant,
};
use winapi::shared::guiddef::GUID;
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
    Adapter(usize, AdapterMessage),

    Nop,
}

pub struct MacSpoof {
    registry_adapters: std::io::Result<Vec<std::io::Result<Adapter>>>,
    connection_manager: std::io::Result<NetConnectionManager>,

    scroll_state: iced::scrollable::State,
}

impl MacSpoof {
    pub fn new() -> Self {
        let connection_manager = NetConnectionManager::new();
        if let Err(e) = connection_manager.as_ref() {
            println!("Failed to create connection manager: {}", e);
        }

        let mut ret = MacSpoof {
            registry_adapters: Err(std::io::Error::from_raw_os_error(0)),
            connection_manager,
            scroll_state: iced::scrollable::State::new(),
        };
        ret.refresh_adapters();
        ret
    }

    pub fn get_network_connections(
        &self,
    ) -> std::io::Result<Vec<(NetConnection, NetConProperties)>> {
        let mut connections = Vec::with_capacity(4);
        // TODO: Handle error instead of ignoring
        if let Ok(connection_manager) = NetConnectionManager::new() {
            // self.connection_manager.as_ref()
            for connection_result in connection_manager.iter()? {
                let connection = connection_result?;
                let properties = connection.get_properties()?;
                connections.push((connection, properties));
            }
        } else {
            println!("NetConnectionManager is in the error state");
        }

        Ok(connections)
    }

    /// Turn an adapter off or on. Returns false if the adapter could not be found.
    pub fn toggle_adapter(&mut self, target: &str, enable: bool) -> std::io::Result<bool> {
        let connections = self.get_network_connections()?;
        let target_connection = connections
            .iter()
            .find(|(_, props)| props.device_name().map(|s| s == target).unwrap_or(false));

        let target_connection = match target_connection {
            Some(connection) => connection,
            None => return Ok(false),
        };

        if enable {
            target_connection.0.connect()?;
        } else {
            target_connection.0.disconnect()?;
        }

        Ok(true)
    }

    pub fn refresh_adapters(&mut self) {
        let start = Instant::now();
        self.registry_adapters = get_registry_adapters().map(|registry_adapters| {
            registry_adapters
                .into_iter()
                .map(|adapter| adapter.map(Adapter::new))
                .collect()
        });
        println!("Got registry adapters in {:?}", start.elapsed());
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
                    Ok(Some(Ok(adapter))) => {
                        let reset_adapter = matches!(message, AdapterMessage::SetHardwareAddress);

                        let command = adapter
                            .update(message, clipboard)
                            .map(move |msg| Message::Adapter(i, msg));

                        if reset_adapter {
                            match adapter.registry_adapter.get_description() {
                                Ok(name) => {
                                    if let Err(e) = self.toggle_adapter(&name, false) {
                                        eprintln!("Failed to turn off adapter: {}", e);
                                    }
                                    if let Err(e) = self.toggle_adapter(&name, true) {
                                        eprintln!("Failed to turn on adapter: {}", e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to get description: {}", e);
                                }
                            }
                        }

                        command
                    }
                    Ok(Some(Err(_e))) => {
                        println!("Cannot process Adapter Message for adapter {} as it is in the error state: {:#?}", i, message);
                        Command::none()
                    }
                    Ok(None) => {
                        println!("Cannot process Adapter Message for adapter {} as it does not exist: {:#?}", i, message);
                        Command::none()
                    }
                    Err(_e) => {
                        println!("`registry_adapters` is in error state. Cannot process Adapter Message for adapter {}: {:#?}", i, message);
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

    Nop,
}

pub struct Adapter {
    registry_adapter: RegistryAdapter,

    hardware_address: String,
    harware_address_state: iced::text_input::State,
}

impl Adapter {
    pub fn new(registry_adapter: RegistryAdapter) -> Self {
        let hardware_address = registry_adapter
            .get_hardware_address()
            .map(|res| {
                res.map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(String::new)
            })
            .unwrap_or_else(|e| e.to_string());
        Adapter {
            registry_adapter,

            hardware_address,
            harware_address_state: iced::text_input::State::new(),
        }
    }

    pub fn update(
        &mut self,
        message: AdapterMessage,
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
                    Some(self.hardware_address.as_str())
                };

                if let Err(e) = self.registry_adapter.set_hardware_address(hardware_address) {
                    // TODO: Give user visual feedback
                    println!("Failed to set hardware address: {}", e);
                } else {
                    println!(
                        "Set Hardware Address to {}",
                        hardware_address.unwrap_or("not set")
                    );
                }

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
            .push(hardware_address);

        column.into()
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

    /// Returns `None` if the hardware address does not exist.
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

    /// Set the hardware address.
    /// Pass `None` to delete the registry key and reset the hardware address to its default.
    pub fn set_hardware_address(&self, hardware_address: Option<&str>) -> std::io::Result<()> {
        match hardware_address {
            Some(hardware_address) => self.key.set_value(Self::HW_ADDRESS_KEY, &hardware_address),
            None => self.key.delete_value(Self::HW_ADDRESS_KEY),
        }
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

fn fmt_guid_to_string(guid: &GUID) -> String {
    format!(
        "{:x}-{:x}-{:x}-{:x}",
        guid.Data1,
        guid.Data2,
        guid.Data3,
        u64::from_le_bytes(guid.Data4)
    )
}
