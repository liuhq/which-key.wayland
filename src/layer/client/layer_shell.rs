use smithay_client_toolkit::{
    delegate_layer,
    reexports::client::{Connection, QueueHandle},
    shell::wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
};

use crate::layer::client::WkLayer;

impl LayerShellHandler for WkLayer {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        _configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw();
        }
    }
}

delegate_layer!(WkLayer);
