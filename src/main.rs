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
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // TODO: Restart the com thread or terminate the program when it exits prematurely.
        // TODO: Gracefully handle com thread not spawning somehow instead of panicking.
        let (com_thread, _com_thread_has_exited) =
            ComThread::new().expect("failed to create com thread");

        let adapters_info = AdaptersInfo::new();
        let mac_spoof = MacSpoof::new(com_thread);
        let resolve_arp = ResolveArp::new();
        let settings = crate::settings::Settings::new();

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

fn main() -> anyhow::Result<()> {
    // Setup colored terminal output
    if let Err(e) = self::settings::set_virtual_terminal_processing(true) {
        // Logging is not set up here so just send to stderr.
        // We do this on the same thread since we haven't launched the gui yet,
        // so there is no eventloop to block.
        eprintln!("failed to set virtual terminal processing: {:?}", e);
    }

    crate::logger::setup().context("failed to setup logger")?;

    let mut settings = Settings::default();
    settings.window.size = (640, 480);
    App::run(settings).context("failed to run app")?;
    // TODO: Figure out a way to make this run. App::run just kills the program.
    // com_thread_handle
    //    .join()
    //    .map_err(|_e| anyhow::anyhow!("com thread error"))?;

    Ok(())
}
