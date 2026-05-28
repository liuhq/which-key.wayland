use clap::Parser;
use which_key_wayland::{Cli, Config, SubCommand, WhichKey, ipc};

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
        None => {
            if ipc::ipc_show() {
                return;
            }
        }
    }

    let config_path = Config::get_path();
    let config = match &config_path {
        Some(p) => Config::load(p),
        None => Config::default(),
    };

    let Some((rx, wake_fd)) = ipc::init() else {
        return;
    };

    let (mut wk_layer, mut event_queue) = WhichKey::new(config, config_path, rx, wake_fd);
    wk_layer.run(&mut event_queue);
}
