use clap::Parser;
use which_key_wayland::{Cli, Config, SubCommand, WhichKey, ipc};

fn main() {
    env_logger::init();

    match Cli::parse().command {
        SubCommand::Show { key } => {
            let (rx, wake_fd) = match ipc::init() {
                Ok(Some((rx, wake_fd))) => (rx, wake_fd),
                Ok(None) => {
                    let handled = match key.as_deref() {
                        Some(key) => ipc::ipc_show_key(key),
                        None => ipc::ipc_show(),
                    };

                    if !handled {
                        log::error!("Failed to send show request to existing instance")
                    }

                    return;
                }
                Err(e) => {
                    log::error!("Failed to initialize IPC: {e:#}");
                    return;
                }
            };

            let config_path = Config::get_path();
            let config = match &config_path {
                Some(p) => Config::load(p),
                None => Config::default(),
            };

            let (mut wk_layer, mut event_queue) = WhichKey::new(config, config_path, rx, wake_fd);

            if let Some(key) = key {
                ipc::ipc_show_key(&key);
            } else {
                ipc::ipc_show();
            }

            wk_layer.run(&mut event_queue);
        }
        SubCommand::Quit => {
            ipc::ipc_quit();
        }
        SubCommand::Reload => {
            ipc::ipc_reload();
        }
    }
}
