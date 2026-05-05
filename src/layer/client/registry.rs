use smithay_client_toolkit::{
    delegate_registry,
    output::OutputState,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
};

use crate::layer::client::WkLayer;

impl ProvidesRegistryState for WkLayer {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_registry!(WkLayer);
