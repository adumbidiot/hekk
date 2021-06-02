use bitflags::bitflags;
use std::os::windows::raw::HANDLE;
use winapi::{
    shared::{
        minwindef::{
            DWORD,
            FALSE,
        },
        windef::HWND,
    },
    um::{
        consoleapi::{
            GetConsoleMode,
            SetConsoleMode,
        },
        handleapi::INVALID_HANDLE_VALUE,
        processenv::GetStdHandle,
        winbase::{
            STD_ERROR_HANDLE,
            STD_INPUT_HANDLE,
            STD_OUTPUT_HANDLE,
        },
        wincon::{
            GetConsoleWindow,
            DISABLE_NEWLINE_AUTO_RETURN,
            ENABLE_ECHO_INPUT,
            ENABLE_INSERT_MODE,
            ENABLE_LINE_INPUT,
            ENABLE_LVB_GRID_WORLDWIDE,
            ENABLE_MOUSE_INPUT,
            ENABLE_PROCESSED_INPUT,
            ENABLE_PROCESSED_OUTPUT,
            ENABLE_QUICK_EDIT_MODE,
            ENABLE_VIRTUAL_TERMINAL_INPUT,
            ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            ENABLE_WINDOW_INPUT,
            ENABLE_WRAP_AT_EOL_OUTPUT,
        },
        winuser::{
            IsWindowVisible,
            ShowWindow,
            SW_HIDE,
            SW_SHOW,
            SW_SHOWNOACTIVATE,
        },
    },
};

#[derive(Debug)]
pub struct ConsoleWindow(HWND);

impl ConsoleWindow {
    /// Get the console window for this process if it exists.
    pub fn get() -> Option<Self> {
        let handle = unsafe { GetConsoleWindow() };
        if handle.is_null() {
            None
        } else {
            Some(Self(handle))
        }
    }

    /// Show console window
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.0, SW_SHOW);
        }
    }

    /// Show console window without activating it
    pub fn show_no_activate(&self) {
        unsafe {
            ShowWindow(self.0, SW_SHOWNOACTIVATE);
        }
    }

    /// Hide console window
    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.0, SW_HIDE);
        }
    }

    /// Whether the window is currently visible
    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.0) != FALSE }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ConsoleHandleType {
    Input,
    Output,
    Error,
}

/// A Handle to a console stream
/// Resource: https://github.com/npocmaka/batch.scripts/blob/master/hybrids/.net/c/quickEdit.bat
#[derive(Debug)]
pub struct ConsoleHandle(HANDLE);

impl ConsoleHandle {
    pub fn get(console_handle_type: ConsoleHandleType) -> std::io::Result<Self> {
        let console_handle_type = match console_handle_type {
            ConsoleHandleType::Input => STD_INPUT_HANDLE,
            ConsoleHandleType::Output => STD_OUTPUT_HANDLE,
            ConsoleHandleType::Error => STD_ERROR_HANDLE,
        };

        let handle = unsafe { GetStdHandle(console_handle_type) };

        if handle == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }

        Ok(Self(handle))
    }

    /// Get the console mode.
    ///
    /// # Panics
    /// Panics if unknown modes are detected.
    pub fn get_mode(&self) -> std::io::Result<ConsoleModeFlags> {
        let mut flags = 0;
        let ret = unsafe { GetConsoleMode(self.0.cast(), &mut flags) };

        if ret == 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(ConsoleModeFlags::from_bits(flags).expect("unknown console mode flag bits detected"))
    }

    /// Set the console mode
    pub fn set_mode(&self, flags: ConsoleModeFlags) -> std::io::Result<()> {
        let ret = unsafe { SetConsoleMode(self.0, flags.bits()) };

        if ret == 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }
}

bitflags! {
    pub struct ConsoleModeFlags: DWORD {
        const ENABLE_ECHO_INPUT = ENABLE_ECHO_INPUT;
        const ENABLE_INSERT_MODE = ENABLE_INSERT_MODE;
        const ENABLE_LINE_INPUT = ENABLE_LINE_INPUT;
        const ENABLE_MOUSE_INPUT = ENABLE_MOUSE_INPUT;
        const ENABLE_PROCESSED_INPUT = ENABLE_PROCESSED_INPUT;
        const ENABLE_QUICK_EDIT_MODE  = ENABLE_QUICK_EDIT_MODE;
        const ENABLE_WINDOW_INPUT = ENABLE_WINDOW_INPUT;
        const ENABLE_VIRTUAL_TERMINAL_INPUT = ENABLE_VIRTUAL_TERMINAL_INPUT;


        const ENABLE_PROCESSED_OUTPUT = ENABLE_PROCESSED_OUTPUT;
        const ENABLE_WRAP_AT_EOL_OUTPUT = ENABLE_WRAP_AT_EOL_OUTPUT;
        const ENABLE_VIRTUAL_TERMINAL_PROCESSING = ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        const DISABLE_NEWLINE_AUTO_RETURN = DISABLE_NEWLINE_AUTO_RETURN;
        const ENABLE_LVB_GRID_WORLDWIDE = ENABLE_LVB_GRID_WORLDWIDE;

        const UNKNOWN_1 = 0x100;
        const UNKNOWN_2 = 0x80;
    }
}
