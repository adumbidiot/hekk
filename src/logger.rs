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

    #[allow(dead_code)]
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

pub fn setup() -> anyhow::Result<()> {
    if let Err(e) = log::set_logger(&*LOGGER) {
        anyhow::bail!("failed to set logger: {}", e);
    }
    log::set_max_level(log::LevelFilter::Info);

    Ok(())
}
