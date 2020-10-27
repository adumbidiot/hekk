#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cmd;
mod types;

use self::cmd::Cmd;
use self::types::AdapterInfo;

fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(2);
    }

    tauri::AppBuilder::new()
        .setup(|webview, _| {
            let mut webview = webview.as_mut();
            let mut webview_clone = webview.clone();

            tauri::event::listen("GetVersion", move |_| {
                dbg!("GetVersion");
                tauri::event::emit(&mut webview_clone, "SetVersion", Some("0.0.0")).unwrap();
            });

            webview
                .dispatch(move |w| {
                    // w.eval("window.onTauriInit()");
                })
                .unwrap();
        })
        .invoke_handler(|webview, arg| match serde_json::from_str(arg) {
            Err(e) => Err(e.to_string()),
            Ok(command) => {
                match command {
                    Cmd::Version { callback, error } => {
                        tauri::execute_promise(webview, move || Ok("0.0.0"), callback, error);
                    }
                    Cmd::GetAdapterInfo { callback, error } => {
                        tauri::execute_promise(
                            webview,
                            move || {
                                iphlpapi::get_adapters_info()
                                    .map(|adapters_info| {
                                        adapters_info
                                            .iter()
                                            .map(AdapterInfo::from)
                                            .collect::<Vec<_>>()
                                    })
                                    .map_err(From::from)
                            },
                            callback,
                            error,
                        );
                    }
                }
                Ok(())
            }
        })
        .build()
        .run();
}
