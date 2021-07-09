mod adapters_info;
mod com_thread;
mod console;
mod logger;
mod mac_spoof;
mod registry_adapter;
mod resolve_arp;
mod settings;
mod style;

pub use crate::console::{
    ConsoleHandle,
    ConsoleHandleType,
    ConsoleModeFlags,
    ConsoleWindow,
};
use crate::{
    adapters_info::AdaptersInfo,
    com_thread::ComThread,
    mac_spoof::MacSpoof,
    resolve_arp::ResolveArp,
    style::GreyStyle,
};
use anyhow::Context;
use iced::{
    Application,
    Clipboard,
    Command,
    Element,
    Length,
    Settings,
};
use iced_aw::TabLabel;
use log::warn;
use macaddr::MacAddr;
use std::{
    convert::TryInto,
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(usize),

    AdaptersInfo(crate::adapters_info::Message),
    MacSpoof(crate::mac_spoof::Message),
    ResolveArp(crate::resolve_arp::Message),
    Settings(crate::settings::Message),

    Nop,
}

pub struct App {
    active_tab: usize,

    adapters_info: crate::adapters_info::AdaptersInfo,
    mac_spoof: crate::mac_spoof::MacSpoof,
    resolve_arp: crate::resolve_arp::ResolveArp,
    settings: crate::settings::Settings,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = UserSettings;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        // TODO: Restart the com thread or terminate the program when it exits prematurely.
        // TODO: Gracefully handle com thread not spawning somehow instead of panicking.
        let (com_thread, _com_thread_has_exited) =
            ComThread::new().expect("failed to create com thread");

        let adapters_info = AdaptersInfo::new();
        let mac_spoof = MacSpoof::new(com_thread);
        let resolve_arp = ResolveArp::new();
        let mut settings = crate::settings::Settings::new();

        // Copy settings
        settings.set_console(flags.console);
        settings.set_debug(flags.debug);

        (
            App {
                active_tab: 0,

                adapters_info,
                mac_spoof,
                resolve_arp,
                settings,
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
            Message::MacSpoof(msg) => self.mac_spoof.update(msg, clipboard).map(Message::MacSpoof),
            Message::ResolveArp(msg) => self
                .resolve_arp
                .update(msg, clipboard)
                .map(Message::ResolveArp),
            Message::Settings(msg) => self.settings.update(msg, clipboard).map(Message::Settings),
            Message::Nop => Command::none(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        iced_aw::Tabs::new(self.active_tab, Message::TabSelected)
            .push(
                TabLabel::Text("Adapter Info".to_string()),
                self.adapters_info.view().map(Message::AdaptersInfo),
            )
            .push(
                TabLabel::Text("Spoof MAC".to_string()),
                self.mac_spoof.view().map(Message::MacSpoof),
            )
            .push(
                TabLabel::Text("Resolve ARP".to_string()),
                self.resolve_arp.view().map(Message::ResolveArp),
            )
            .push(
                TabLabel::Text("Settings".to_string()),
                self.settings.view().map(Message::Settings),
            )
            .tab_bar_style(GreyStyle)
            // .icon_font(ICON_FONT)
            .width(Length::Fill)
            .height(Length::Fill)
            .tab_bar_position(iced_aw::TabBarPosition::Top)
            .into()
    }
}

fn format_mac_address(mut f: impl std::fmt::Write, data: &[u8]) -> std::fmt::Result {
    let v6_addr: Result<[u8; 6], _> = data.try_into();
    if let Ok(addr) = v6_addr {
        return write!(f, "{:-}", MacAddr::from(addr));
    }

    let v8_addr: Result<[u8; 8], _> = data.try_into();
    if let Ok(addr) = v8_addr {
        return write!(f, "{:-}", MacAddr::from(addr));
    }

    // Backup fmt
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

fn format_mac_address_to_string(address: &[u8]) -> String {
    // Each u8 maps to 3 ascii chars, "XX-", except the last one.
    // We overallocate for simplicity.
    let mut ret = String::with_capacity(address.len() * 3);
    format_mac_address(&mut ret, address).expect("failed to format hardware address");
    ret
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UserSettings {
    pub debug: bool,
    pub console: bool,
}

impl UserSettings {
    pub fn new() -> Self {
        Self {
            debug: false,
            console: true,
        }
    }

    pub fn data_dir() -> anyhow::Result<PathBuf> {
        let path = skylight::get_known_folder_path(skylight::FolderId::LocalAppData)
            .context("failed to get local app data folder")?
            .as_os_string();
        let path = PathBuf::from(path).join("Hekk");
        Ok(path)
    }

    pub fn settings_path() -> anyhow::Result<PathBuf> {
        Ok(Self::data_dir()?.join("settings.toml"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::settings_path()?;
        let data = std::fs::read_to_string(path).context("failed to read data")?;
        toml::from_str(&data).context("failed to deserialize data")
    }

    pub fn save(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(Self::data_dir()?).context("failed to create data dir")?;

        let path = Self::settings_path()?;
        let data = toml::to_string_pretty(self).context("failed to serialize")?;
        std::fs::write(path, data).context("failed to write")?;
        Ok(())
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    // Setup
    {
        // Setup colored terminal output
        if let Err(e) = self::settings::set_virtual_terminal_processing(true)
            .context("failed to set up virtual terminal processing")
        {
            // Logging is not set up here so just send to stderr.
            // We do this on the same thread since we haven't launched the gui yet,
            // so there is no eventloop to block.
            eprintln!("failed to set virtual terminal processing: {:?}", e);
        }
    }

    // TODO: Consider catching panics
    let exit_code = match real_main() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{:?}", e);
            1
        }
    };

    // This actually will only exit on error since winit decides to exit with 0 for us on success.
    std::process::exit(exit_code);
}

fn real_main() -> anyhow::Result<()> {
    // Setup logger
    crate::logger::setup().context("failed to setup logger")?;

    // Load settings
    let user_settings = match UserSettings::load().context("failed to load user settings") {
        Ok(settings) => settings,
        Err(e) => {
            warn!("{:?}", e);
            warn!("Using default settings...");
            Default::default()
        }
    };

    let mut settings = Settings::with_flags(user_settings);
    settings.window.size = (640, 480);
    App::run(settings).context("failed to run app")?;
    // TODO: Figure out a way to make this run. App::run just kills the program.
    // com_thread_handle
    //    .join()
    //    .map_err(|_e| anyhow::anyhow!("com thread error"))?;

    Ok(())
}
