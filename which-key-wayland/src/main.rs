use clap::Parser;
use which_key_wayland::{Cli, SubCommand, WhichKey, config_parse, ipc};

fn main() {
    env_logger::init();

    match Cli::parse().command {
        Some(SubCommand::Show) => {
            ipc::ipc_show();
            return;
        }
        Some(SubCommand::Quit) => {
            ipc::ipc_quit();
            return;
        }
        None => {}
    }

    let config = match std::fs::read_to_string("./examples/config.kdl") {
        Ok(raw) => match config_parse(&raw) {
            Ok(config) => config,
            Err(err) => {
                log::error!("{err}");
                return;
            }
        },
        Err(err) => {
            log::error!("{err}");
            return;
        }
    };

    let Some((rx, wake_fd)) = ipc::init() else {
        return;
    };

    let (mut wk_layer, mut event_queue) = WhichKey::new(config, rx, wake_fd);
    wk_layer.run(&mut event_queue);
}
