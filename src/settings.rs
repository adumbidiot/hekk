use crate::{
    ConsoleWindow,
    GreyStyle,
};
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

#[derive(Debug, Clone)]
pub enum Message {
    ConsoleToggled(bool),
}

pub struct Settings {
    console: Option<ConsoleWindow>,

    scroll_state: iced::scrollable::State,
}

impl Settings {
    pub fn new() -> Self {
        let console = ConsoleWindow::get();

        /*
        let console_handle = ConsoleHandle::get(ConsoleHandleType::Input);
        match console_handle {
            Ok(console_handle) => {
                match console_handle.get_mode() {
                    Ok(mut mode) => {
                        mode.remove(ConsoleModeFlags::ENABLE_QUICK_EDIT_MODE);
                        if let Err(e) = console_handle.set_mode(mode) {
                            eprintln!("failed to set console input mode: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("failed to get console input mode: {}", e);
                    }
                };
            }
            Err(e) => {
                eprintln!("failed to get console handle: {}", e);
            }
        }
        */

        Settings {
            console,

            scroll_state: iced::scrollable::State::new(),
        }
    }

    pub fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::ConsoleToggled(show) => {
                if let Some(console) = self.console.as_ref() {
                    if show {
                        console.show();
                    } else {
                        console.hide();
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
