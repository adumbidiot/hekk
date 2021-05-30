use crate::GreyStyle;
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
    Nop,
}

pub struct MacSpoof {
    registry_adapters: std::io::Result<Vec<std::io::Result<RegistryAdapter>>>,

    scroll_state: iced::scrollable::State,
}

impl MacSpoof {
    pub fn new() -> Self {
        let mut ret = MacSpoof {
            registry_adapters: Err(std::io::Error::from_raw_os_error(0)),
            scroll_state: iced::scrollable::State::new(),
        };
        ret.refresh_adapters();
        ret
    }

    pub fn refresh_adapters(&mut self) {
        let start = Instant::now();
        self.registry_adapters = get_registry_adapters();
        println!("Got registry adapters in {:?}", start.elapsed());
    }

    pub fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Nop => Command::none(),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let title = Text::new("Mac Spoof").size(36);
        let mut column = Column::new().spacing(10).push(title);

        match self.registry_adapters.as_ref() {
            Ok(registry_adapters) => {
                for (i, registry_adapter) in registry_adapters.iter().enumerate() {
                    let info: Element<_> = match registry_adapter {
                        Ok(registry_adapter) => {
                            let hardware_address = Text::new(format!(
                                "Hardware Address: {}",
                                registry_adapter
                                    .get_hardware_address()
                                    .map(|res| res
                                        .map(|s| s.to_string_lossy().into_owned())
                                        .unwrap_or_else(|| "Not Set".to_string()))
                                    .unwrap_or_else(|e| e.to_string())
                            ))
                            .size(15);

                            let column = Column::new()
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
                                .push(hardware_address);

                            column.into()
                        }
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
