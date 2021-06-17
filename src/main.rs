mod adapters_info;
mod com_thread;
mod console;
mod logger;
mod mac_spoof;
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
    Settings(crate::settings::Message),

    Nop,
}

pub struct App {
    active_tab: usize,

    adapters_info: AdaptersInfo,
    mac_spoof: MacSpoof,
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
        let settings = crate::settings::Settings::new();

        (
            App {
                active_tab: 0,

                adapters_info,
                mac_spoof,
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

fn main() -> anyhow::Result<()> {
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
