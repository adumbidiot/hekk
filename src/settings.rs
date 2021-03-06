use crate::{
    ConsoleHandle,
    ConsoleHandleType,
    ConsoleModeFlags,
    ConsoleWindow,
    GreyStyle,
};
use anyhow::Context;
use iced::{
    Checkbox,
    Clipboard,
    Column,
    Command,
    Container,
    Element,
    Length,
    Scrollable,
    Text,
};
use log::{
    error,
    info,
    warn,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    ConsoleToggled(bool),
    DebugToggled(bool),

    SaveResult(Arc<anyhow::Result<()>>),
}

pub struct Settings {
    console: Option<ConsoleWindow>,
    debug: bool,

    scroll_state: iced::scrollable::State,
}

impl Settings {
    pub fn new() -> Self {
        let console = ConsoleWindow::get();

        // We currently use a logging thread that buffers console output, removing the need to work around quick edit mode.
        // This is just a filler.
        // Maybe we can add a setting for the user to set the console mode later.
        let quick_edit_mode = None;
        if let Some(quick_edit_mode) = quick_edit_mode {
            if let Err(e) = set_quick_edit_mode(quick_edit_mode) {
                warn!("failed to set quick edit mode: {:?}", e);
            }
        }

        Settings {
            console,
            debug: false,

            scroll_state: iced::scrollable::State::new(),
        }
    }

    /// Set whether the console is shown.
    ///
    /// This is a nop if the console window could not be aquired.
    pub fn set_console(&mut self, show: bool) {
        if let Some(console) = self.console.as_ref() {
            if show {
                console.show_no_activate();
            } else {
                console.hide();
            }
        }
    }

    /// Set whether debug info is printed.
    pub fn set_debug(&mut self, debug: bool) {
        // TODO: Move to logger so that all data is in one place.
        if debug {
            log::set_max_level(log::LevelFilter::Debug);
        } else {
            log::set_max_level(log::LevelFilter::Info);
        }
        self.debug = debug;
    }

    pub fn save_settings_command(&self) -> Command<Message> {
        let data = crate::UserSettings {
            console: self.console.as_ref().map_or(true, |c| c.is_visible()),
            debug: self.debug,
        };

        Command::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    data.save().context("failed to save user settings")
                })
                .await
                .context("tokio task panicked")??;
                Ok(())
            },
            |r| Message::SaveResult(Arc::new(r)),
        )
    }

    pub fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        // TODO: ratelimit saves somehow
        match message {
            Message::ConsoleToggled(show) => {
                self.set_console(show);
                self.save_settings_command()
            }
            Message::DebugToggled(debug) => {
                self.set_debug(debug);
                self.save_settings_command()
            }
            Message::SaveResult(r) => {
                match r.as_ref() {
                    Ok(()) => {
                        info!("Saved user settings");
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
                Command::none()
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut column = Column::new()
            .spacing(10)
            .push(Text::new("Settings").size(36));

        if let Some(console) = self.console.as_ref() {
            column = column.push(Checkbox::new(
                console.is_visible(),
                "Show Console",
                Message::ConsoleToggled,
            ));
        }

        column = column.push(Checkbox::new(self.debug, "Debug", Message::DebugToggled));

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

/// Set the quick edit mode of the attached console.
///
/// Disabling it prevents hangs when writing if the user is interacting with the console,
/// but the user is unable to copy text from it.
/// A hang will crash the program as the program is left unable to deal with window messages.
/// This cannot be easily worked around as it seems like the kernel call itself hangs for the entire duration of the user interaction.
/// Good Job Microsoft!
pub fn set_quick_edit_mode(active: bool) -> anyhow::Result<()> {
    let console_handle =
        ConsoleHandle::get(ConsoleHandleType::Input).context("failed to get console handle")?;
    let mut mode = console_handle
        .get_mode()
        .context("failed to get console input mode")?;
    if active {
        mode.insert(ConsoleModeFlags::ENABLE_QUICK_EDIT_MODE);
    } else {
        mode.remove(ConsoleModeFlags::ENABLE_QUICK_EDIT_MODE);
    }
    console_handle
        .set_mode(mode)
        .context("failed to set console input mode")?;
    Ok(())
}

pub fn set_virtual_terminal_processing(active: bool) -> anyhow::Result<()> {
    let console_handle = ConsoleHandle::get(ConsoleHandleType::Output)
        .context("failed to get output console handle")?;
    let mut mode = console_handle
        .get_mode()
        .context("failed to get console output mode")?;
    if active {
        mode.insert(ConsoleModeFlags::ENABLE_VIRTUAL_TERMINAL_PROCESSING);
    } else {
        mode.remove(ConsoleModeFlags::ENABLE_VIRTUAL_TERMINAL_PROCESSING);
    }
    console_handle
        .set_mode(mode)
        .context("failed to set console output mode")?;
    Ok(())
}
