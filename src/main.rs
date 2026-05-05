use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{Connection, globals::registry_queue_init},
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::{KeyboardInteractivity, Layer, LayerShell},
    },
    shm::{Shm, slot::SlotPool},
};
use which_key_wayland::{config::parser::config_parse, layer::client::WkLayer};

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr_layer_shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("simple_layer"), None);

    // NOTE: example config
    match std::fs::read_to_string("./examples/config.kdl") {
        Ok(raw) => match config_parse(&raw) {
            Ok(config) => {
                layer.set_anchor(config.layout.anchor);
                layer.set_margin(
                    config.layout.margin.top,
                    config.layout.margin.right,
                    config.layout.margin.bottom,
                    config.layout.margin.left,
                );
                layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
                layer.set_size(config.layout.width, 1);
                layer.commit();

                let pool = SlotPool::new((config.layout.width * 4) as usize, &shm)
                    .expect("Failed to create pool");

                let mut wk_layer = WkLayer {
                    registry_state: RegistryState::new(&globals),
                    output_state: OutputState::new(&globals, &qh),
                    seat_state: SeatState::new(&globals, &qh),
                    shm,
                    pool,
                    buffer: None,
                    layer,
                    exit: false,
                    first_configure: true,
                    shift: None,
                    keyboard: None,
                    keyboard_focus: false,
                    config,
                };

                loop {
                    event_queue.blocking_dispatch(&mut wk_layer).unwrap();

                    if wk_layer.exit {
                        println!("exiting wk_layer");
                        break;
                    }
                }
            }
            Err(err) => eprintln!("{err}"),
        },
        Err(err) => eprintln!("{err}"),
    };
}
