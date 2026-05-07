use which_key_wayland::{config::parser::config_parse, layer::client::WkLayer};

fn main() {
    env_logger::init();

    // NOTE: example config
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

    let (mut wk_layer, mut event_queue) = WkLayer::new(config);
    wk_layer.run(&mut event_queue);
}
