use anyhow::Context;
use log::{
    debug,
    info,
    warn,
};
use netcon::{
    NetConProperties,
    NetConnection,
    NetConnectionManager,
};
use std::{
    ffi::{
        OsStr,
        OsString,
    },
    time::Instant,
};
use uuid::Uuid;
use winapi::shared::guiddef::GUID;

const MAX_BUFFERED_COMMANDS: usize = 32;

pub type ComThreadHasExited = tokio::sync::oneshot::Receiver<anyhow::Result<()>>;
pub type ComThreadResultSender<T> = tokio::sync::oneshot::Sender<T>;

#[derive(Debug)]
enum ComCommand {
    ResetNetworkConnection {
        adapter_name: String,
        adapter_description: OsString,
        responder: ComThreadResultSender<anyhow::Result<()>>,
    },
}

/// A COM thread handle. It can be used to issue commands to the COM thread.
///
/// This app proxies COM functions to this thread for 2 purposes:
/// 1. To not block the UI thread, as a lot of COM operations can take seconds to complete
/// 2. To allow winit to be the brutal overlord that it is, as it nukes the multithreaded com apartments that I try to set up
#[derive(Debug, Clone)]
pub struct ComThread {
    command_tx: tokio::sync::mpsc::Sender<ComCommand>,
}

impl ComThread {
    pub fn new() -> anyhow::Result<(Self, ComThreadHasExited)> {
        let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(MAX_BUFFERED_COMMANDS);
        let (exited_tx, exited_rx) = tokio::sync::oneshot::channel();

        std::thread::Builder::new()
            .name("com-thread".into())
            .spawn(move || {
                info!("Starting COM thread");
                if let Err(e) =
                    skylight::init_mta_com_runtime().context("failed to init mta com runtime")
                {
                    let _ = exited_tx.send(Err(e)).is_ok();
                    return;
                }

                let connection_manager = match NetConnectionManager::new()
                    .context("failed to create connection manager")
                {
                    Ok(connection_manager) => connection_manager,
                    Err(e) => {
                        let _ = exited_tx.send(Err(e)).is_ok();
                        return;
                    }
                };

                while let Some(command) = command_rx.blocking_recv() {
                    process_command(&connection_manager, command);
                }

                info!("Shutting down COM thread");
                let _ = exited_tx.send(Ok(())).is_ok();
            })
            .context("failed to spawn com thread")?;

        Ok((Self { command_tx }, exited_rx))
    }

    /// Reset the network adapter.
    /// Returns an error if the adpater could not be located or restarted.
    pub async fn reset_network_connection(
        &self,
        adapter_name: String,
        adapter_description: OsString,
    ) -> anyhow::Result<()> {
        let start = Instant::now();
        let (responder, rx) = tokio::sync::oneshot::channel();
        self.command_tx
            .send(ComCommand::ResetNetworkConnection {
                adapter_name: adapter_name.clone(),
                adapter_description: adapter_description.clone(),
                responder,
            })
            .await
            .context("failed to send request")?;
        let result = rx.await.context("failed to receive result")?;
        info!(
            "Reset network adapter '{}' | '{}' in {:?}",
            adapter_name,
            adapter_description.to_string_lossy(),
            start.elapsed()
        );

        result
    }
}

fn process_command(connection_manager: &NetConnectionManager, command: ComCommand) {
    match command {
        ComCommand::ResetNetworkConnection {
            adapter_name,
            adapter_description,
            responder,
        } => {
            let result =
                reset_network_connection(connection_manager, &adapter_name, &adapter_description);
            let _ = responder.send(result).is_ok();
        }
    }
}

fn reset_network_connection(
    connection_manager: &NetConnectionManager,
    adapter_name: &str,
    adapter_description: &OsStr,
) -> anyhow::Result<()> {
    let (connection, _properties) =
        find_network_connection(connection_manager, adapter_name, adapter_description)
            .context("failed to get network connection")?
            .context("failed to find network connection")?;

    connection.disconnect()?;
    connection.connect()?;

    Ok(())
}

fn find_network_connection(
    connection_manager: &NetConnectionManager,
    adapter_name: &str,
    adapter_description: &OsStr,
) -> std::io::Result<Option<(NetConnection, NetConProperties)>> {
    let adapter_name = adapter_name.trim_start_matches('{').trim_end_matches('}');
    debug!("Locating network connection '{}'", adapter_name);

    // Store all connections and properties in buffer
    let mut connections = Vec::with_capacity(32);
    for connection_result in connection_manager.iter()? {
        let connection = connection_result?;
        let properties = connection.get_properties()?;

        // TODO: add wrapper to format wide slice without allocating
        let formatted_guid = fmt_guid_to_string(properties.guid());
        debug!(
            "Located '{}' | '{}' | '{}'",
            properties.name().to_string_lossy(),
            formatted_guid,
            properties.device_name().to_string_lossy(),
        );

        connections.push((connection, properties));
    }

    // 1st pass guid compare
    for i in 0..connections.len() {
        let (_connection, properties) = &connections[i];
        // Adapter names have the form {<guid>}.
        let formatted_guid = fmt_guid_to_string(properties.guid());
        if formatted_guid == adapter_name {
            return Ok(Some(connections.swap_remove(i)));
        }
    }

    warn!(
        "Failed to locate '{}', comparing descriptions...",
        adapter_name
    );

    // 2nd pass description compare
    for i in 0..connections.len() {
        let (_connection, properties) = &connections[i];
        if properties.device_name() == adapter_description {
            return Ok(Some(connections.swap_remove(i)));
        }
    }

    Ok(None)
}

pub fn fmt_guid_to_string(guid: &GUID) -> String {
    Uuid::from_fields(guid.Data1, guid.Data2, guid.Data3, &guid.Data4)
        .expect("a guid was not a valid uuid")
        .to_hyphenated()
        .encode_upper(&mut Uuid::encode_buffer())
        .to_string()
}
