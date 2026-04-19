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
use which_key_wayland::{
    config::{Config, parser::config_parse},
    layer::client::WkLayer,
};

fn main() {
    env_logger::init();

    // TODO: test config parser
    {
        match std::fs::read_to_string("./examples/config.kdl") {
            Ok(raw) => {
                let _ = config_parse(&raw);
            }
            Err(err) => eprintln!("{err}"),
        };
    }

    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("wlr_layer_shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("simple_layer"), None);

    let mock = Config::mock();

    layer.set_anchor(mock.layout.anchor);
    layer.set_margin(
        mock.layout.margin.top,
        mock.layout.margin.right,
        mock.layout.margin.bottom,
        mock.layout.margin.left,
    );
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer.set_size(mock.layout.width, mock.layout.max_height);
    layer.commit();

    let pool = SlotPool::new(
        (mock.layout.width * 4 * mock.layout.max_height) as usize,
        &shm,
    )
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
        config: mock,
    };

    loop {
        event_queue.blocking_dispatch(&mut wk_layer).unwrap();

        if wk_layer.exit {
            println!("exiting wk_layer");
            break;
        }
    }
}
