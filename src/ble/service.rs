use esp_idf_svc::bt::ble::gatt::GattServiceId;
use std::{ marker::PhantomData, sync::Arc };
use super::{ ReadExt, WriteExt };

#[derive(Debug, Clone)]
pub struct Service<State: Sync + Send + Clone = ()> {
    pub service_id: GattServiceId,
    pub num_handles: u16,
    pub read_characteristics: Vec<Arc<dyn ReadExt<State = State>>>,
    pub write_characteristics: Vec<Arc<dyn WriteExt<State = State>>>,
    _p: std::marker::PhantomData<State>,
}

// unsafe impl<T: Sync + Send + Clone> Send for Service<T> {}
// unsafe impl<T: Sync + Send + Clone> Sync for Service<T> {}

impl<T: Sync + Send + Clone> Service<T> {
    pub fn new(service_id: GattServiceId, num_handles: u16) -> Self {
        Self {
            service_id,
            num_handles,
            read_characteristics: Vec::new(),
            write_characteristics: Vec::new(),
            _p: PhantomData,
        }
    }

    pub fn add_read_characteristic(&mut self, characteristic: Arc<dyn ReadExt<State = T>>) {
        self.read_characteristics.push(characteristic);
    }

    pub fn add_write_characteristic(&mut self, characteristic: Arc<dyn WriteExt<State = T>>) {
        self.write_characteristics.push(characteristic);
    }
}
