use anyhow::Context;
use netcon::{
    NetConnection,
    NetConnectionManager,
};
use std::time::Instant;
use winapi::shared::guiddef::GUID;

pub type ComThreadHasExited = tokio::sync::oneshot::Receiver<anyhow::Result<()>>;
pub type ComThreadResultSender<T> = tokio::sync::oneshot::Sender<T>;

#[derive(Debug)]
enum ComCommand {
    ResetNetworkConnection {
        adapter_name: String,
        responder: ComThreadResultSender<anyhow::Result<bool>>,
    },
}

#[derive(Debug, Clone)]
pub struct ComThread {
    command_tx: tokio::sync::mpsc::Sender<ComCommand>,
}

impl ComThread {
    pub fn new() -> (Self, ComThreadHasExited) {
        let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(32);
        let (exited_tx, exited_rx) = tokio::sync::oneshot::channel();

        std::thread::spawn(move || {
            println!("Starting COM thread");
            if let Err(e) =
                skylight::init_mta_com_runtime().context("failed to init mta com runtime")
            {
                let _ = exited_tx.send(Err(e)).is_ok();
                return;
            }

            let connection_manager =
                match NetConnectionManager::new().context("failed to create connection manager") {
                    Ok(connection_manager) => connection_manager,
                    Err(e) => {
                        let _ = exited_tx.send(Err(e)).is_ok();
                        return;
                    }
                };

            while let Some(command) = command_rx.blocking_recv() {
                process_command(&connection_manager, command);
            }

            println!("Shutting down COM thread");
            let _ = exited_tx.send(Ok(())).is_ok();
        });

        (Self { command_tx }, exited_rx)
    }

    /// Reset the network adapter.
    /// Returns an error if the adpater could not be located.
    /// Returns false if the adapter could not be restarted.
    pub async fn reset_network_connection(&self, adapter_name: String) -> anyhow::Result<bool> {
        let start = Instant::now();
        let (responder, rx) = tokio::sync::oneshot::channel();
        self.command_tx
            .send(ComCommand::ResetNetworkConnection {
                adapter_name: adapter_name.clone(),
                responder,
            })
            .await
            .context("failed to send request")?;
        let result = rx.await.context("failed to receive result")?;
        println!(
            "Reset network adapter '{}' in {:?}",
            adapter_name,
            start.elapsed()
        );

        result
    }
}

fn process_command(connection_manager: &NetConnectionManager, command: ComCommand) {
    match command {
        ComCommand::ResetNetworkConnection {
            adapter_name,
            responder,
        } => {
            let result = reset_network_connection(connection_manager, &adapter_name);
            let _ = responder.send(result).is_ok();
        }
    }
}

fn reset_network_connection(
    connection_manager: &NetConnectionManager,
    device_name: &str,
) -> anyhow::Result<bool> {
    let connection = find_network_connection(&connection_manager, device_name)
        .context("failed to get network connection")?
        .context("failed to find network connection")?;

    let mut is_success = true;

    if let Err(e) = connection.disconnect() {
        println!("Failed to disable connection: {}", e);
        is_success = false;
    }

    if let Err(e) = connection.connect() {
        println!("Failed to enable connection: {}", e);
        is_success = false;
    }

    Ok(is_success)
}

fn find_network_connection(
    connection_manager: &NetConnectionManager,
    adapter_name: &str,
) -> std::io::Result<Option<NetConnection>> {
    for connection_result in connection_manager.iter()? {
        let connection = connection_result?;
        let properties = connection.get_properties()?;

        let formatted_guid = format!("{{{}}}", fmt_guid_to_string(properties.guid()));
        if formatted_guid == adapter_name {
            return Ok(Some(connection));
        }
    }

    Ok(None)
}

pub fn fmt_guid_to_string(guid: &GUID) -> String {
    format!(
        "{:X}-{:X}-{:X}-{:X}{:X}-{:X}{:X}{:X}{:X}{:X}{:X}",
        guid.Data1,
        guid.Data2,
        guid.Data3,
        guid.Data4[0],
        guid.Data4[1],
        guid.Data4[2],
        guid.Data4[3],
        guid.Data4[4],
        guid.Data4[5],
        guid.Data4[6],
        guid.Data4[7],
    )
}
