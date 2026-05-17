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
        None => {}
    }

    let config = Config::init();

    let Some((rx, wake_fd)) = ipc::init() else {
        return;
    };

    let (mut wk_layer, mut event_queue) = WhichKey::new(config, rx, wake_fd);
    wk_layer.run(&mut event_queue);
}
