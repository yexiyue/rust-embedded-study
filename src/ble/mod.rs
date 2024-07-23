use esp_idf_svc::bt::{
    ble::{ gap::EspBleGap, gatt::{ server::EspGatts, GattCharacteristic, GattDescriptor } },
    Ble,
    BtDriver,
};
use std::{ fmt::Debug, sync::Arc };
mod service;
mod app;
pub use service::Service;
pub use app::*;
mod app_builder;

type ExBtDriver<'a> = BtDriver<'a, Ble>;

type ExEspBleGap<'a> = Arc<EspBleGap<'a, Ble, Arc<ExBtDriver<'a>>>>;
type ExEspGatts<'a> = Arc<EspGatts<'a, Ble, Arc<ExBtDriver<'a>>>>;

pub trait CharacteristicExt: Debug + Sync + Send {
    fn characteristic(&self) -> GattCharacteristic;
    fn descriptors(&self) -> Vec<GattDescriptor> {
        vec![]
    }
}

pub trait ReadExt: CharacteristicExt {
    type State: Sync + Send + Clone;
    fn on_read(&self, state: Self::State) -> anyhow::Result<&[u8]>;
}

pub trait WriteExt: CharacteristicExt {
    type State: Sync + Send + Clone;
    fn on_write(&self, state: Self::State, data: &[u8]) -> anyhow::Result<()>;
}

pub trait NotifyExt: CharacteristicExt {
    type State: Sync + Send + Clone;
    fn on_subscribe(&self, state: Self::State) -> anyhow::Result<()>;
    fn on_unsubscribe(&self, state: Self::State) -> anyhow::Result<()>;
}

pub fn start<State: Send + Sync + Clone + 'static>(
    ble_app: BLEApp<'static, State>
) -> anyhow::Result<()> {
    let gap = ble_app.gap.clone();
    let gatts = ble_app.gatts.clone();
    let app_id = ble_app.app_id.clone();
    let app = ble_app.clone();
    gap.subscribe(move |event| {
        log::info!("Handled gatts event {event:#?}");
        match app.on_gap_event(event) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to handle gap event: {}", e);
            }
        }
    })?;
    let app = ble_app.clone();
    gatts.subscribe(move |(gatt_if, event)| {
        log::info!("Handled gatts event {event:#?}");
        match app.on_gatts_event(gatt_if, event) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Failed to handle gatts event: {}", e);
            }
        }
    })?;
    gatts.register_app(app_id)?;
    Ok(())
}
