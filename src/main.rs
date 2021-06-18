mod adapters_info;
mod com_thread;
mod console;
mod mac_spoof;
mod registry_adapter;
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
use once_cell::sync::Lazy;

pub struct ThreadLogger {
    sender: crossbeam_channel::Sender<(log::Level, String)>,

    handle: std::thread::JoinHandle<()>,
}

impl ThreadLogger {
    pub fn new() -> Self {
        fn process_message((level, msg): (log::Level, &str)) {
            match level {
                log::Level::Info => {
                    print!("\x1B[96m");
                }
                log::Level::Error => {
                    print!("\x1B[91m");
                }
                log::Level::Warn => {
                    print!("\x1B[93m");
                }
                _ => {}
            }
            print!("[{}] ", level);
            print!("\x1B[0m");

            println!("{}", msg);
        }

        let (tx, rx) = crossbeam_channel::unbounded::<(log::Level, String)>();
        let handle = std::thread::spawn(move || {
            process_message((log::Level::Info, "Starting logger thread"));

            for (level, msg) in rx {
                process_message((level, msg.as_str()))
            }

            process_message((log::Level::Info, "Shutting down logger thread"));
        });

        Self { sender: tx, handle }
    }

    pub fn close(self) -> anyhow::Result<()> {
        drop(self.sender);
        self.handle
            .join()
            .map_err(|_e| anyhow::anyhow!("logger thread panicked"))?;
        Ok(())
    }
}

impl log::Log for ThreadLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.target().starts_with("hekk")
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        self.sender
            .send((record.level(), format!("{}", record.args())))
            .expect("failed to send message to logger thread");
    }

    fn flush(&self) {}
}

impl Default for ThreadLogger {
    fn default() -> Self {
        Self::new()
    }
}

static LOGGER: Lazy<ThreadLogger> = Lazy::new(ThreadLogger::new);

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
    if let Err(e) = log::set_logger(&*LOGGER) {
        anyhow::bail!("failed to set logger: {}", e);
    }
    log::set_max_level(log::LevelFilter::Info);

    let mut settings = Settings::default();
    settings.window.size = (640, 480);
    App::run(settings).context("failed to run app")?;
    // TODO: Figure out a way to make this run. App::run just kills the program.
    // com_thread_handle
    //    .join()
    //    .map_err(|_e| anyhow::anyhow!("com thread error"))?;

    Ok(())
}
