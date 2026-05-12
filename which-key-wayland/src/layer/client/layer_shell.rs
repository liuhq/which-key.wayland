use smithay_client_toolkit::{
    delegate_layer,
    reexports::client::{Connection, QueueHandle},
    shell::wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
};

use std::time::Instant;

use crate::{keybind::page::PageDirection, layer::client::WhichKey};

impl LayerShellHandler for WhichKey {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.hide_overlay();
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        _configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if self.first_configure {
            self.first_configure = false;
            self.draw(None, PageDirection::Forward);
            self.last_key_time = Some(Instant::now());
        }
    }
}

delegate_layer!(WhichKey);
