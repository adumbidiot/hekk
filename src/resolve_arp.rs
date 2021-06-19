use crate::{
    format_mac_address,
    style::{
        ForegroundGreenTextInputStyle,
        GreyStyleCopyTextHack,
    },
    GreyStyle,
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
    Text,
    TextInput,
};
use log::{
    error,
    info,
    warn,
};
use std::{
    convert::TryInto,
    net::Ipv4Addr,
    sync::Arc,
    time::Instant,
};

#[derive(Debug, Clone)]
pub enum Message {
    Nop,

    UpdateIp(String),
    ResolveArp,
    ResolveArpComplete(Arc<anyhow::Result<(u64, usize)>>),
}

pub struct ResolveArp {
    text_input_state: iced::text_input::State,
    resolved_mac_state: iced::text_input::State,

    ip_address: String,
    resolved_mac: String,
}

impl ResolveArp {
    pub fn new() -> Self {
        Self {
            text_input_state: iced::text_input::State::new(),
            resolved_mac_state: iced::text_input::State::new(),

            ip_address: String::new(),
            resolved_mac: String::new(),
        }
    }

    pub fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Nop => Command::none(),
            Message::UpdateIp(ip_address) => {
                self.ip_address = ip_address;
                Command::none()
            }
            Message::ResolveArp => {
                let ip_address = self.ip_address.clone();
                Command::perform(
                    async move { resolve_arp(&ip_address).await.context("failed to resolve") },
                    |res| Message::ResolveArpComplete(Arc::new(res)),
                )
            }
            Message::ResolveArpComplete(res) => {
                match res.as_ref() {
                    Ok((mac, len)) => {
                        self.resolved_mac.clear();
                        if let Err(e) =
                            format_mac_address(&mut self.resolved_mac, &mac.to_ne_bytes()[..*len])
                                .context("failed to format MAC address")
                        {
                            error!("{:?}", e);
                        }

                        if *len == 0 {
                            warn!("mac length is 0");
                        }
                    }
                    Err(e) => {
                        // TODO: Visual Feedback
                        error!("{:?}", e);
                    }
                }

                Command::none()
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let title = Text::new("Resolve ARP").size(36);
        let column = Column::new()
            .spacing(10)
            .push(title)
            .push(
                TextInput::new(
                    &mut self.text_input_state,
                    "Enter ipv4 address",
                    &self.ip_address,
                    Message::UpdateIp,
                )
                .on_submit(Message::ResolveArp)
                .style(ForegroundGreenTextInputStyle)
                .size(15)
                .padding(2),
            )
            .push(
                Row::new().push(Text::new("Resolved MAC: ").size(15)).push(
                    TextInput::new(&mut self.resolved_mac_state, "", &self.resolved_mac, |_| {
                        Message::Nop
                    })
                    .style(GreyStyleCopyTextHack)
                    .size(15),
                ),
            );

        Container::new(Container::new(column).padding(20))
            .style(GreyStyle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

pub async fn resolve_arp(ip_address: &str) -> anyhow::Result<(u64, usize)> {
    let ip = ip_address
        .parse::<Ipv4Addr>()
        .context("invalid ipv4 address")?;

    let start = Instant::now();
    let response = tokio::task::spawn_blocking(move || iphlpapi::send_arp(ip, None))
        .await
        .context("tokio task failed to join")?
        .context("failed to resolve arp");

    info!("Executed `send_arp({})` in {:?}", ip, start.elapsed());

    let (mac, mac_len) = response?;
    let mac_len: usize = mac_len
        .try_into()
        .context("mac_len cannot fit in a usize")?;

    Ok((mac, mac_len))
}
