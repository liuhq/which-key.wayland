use smithay_client_toolkit::{
    delegate_shm,
    shm::{Shm, ShmHandler},
};

use crate::layer::client::WkLayer;

impl ShmHandler for WkLayer {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_shm!(WkLayer);
