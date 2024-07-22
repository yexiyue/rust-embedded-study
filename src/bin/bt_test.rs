use std::sync::Arc;

use enumset::enum_set;
use esp_idf_svc::{
    bt::{
        ble::{
            gap::AdvConfiguration,
            gatt::{ AutoResponse, GattCharacteristic, GattId, GattServiceId, Permission, Property },
        },
        Ble,
        BtDriver,
        BtUuid,
    },
    hal::delay::FreeRtos,
};
use rust_embedded_study::{ ble::{ self, CharacteristicExt, ReadExt, Service, WriteExt }, init };

#[derive(Debug, Clone, Default)]
struct TestReadWrite;

impl CharacteristicExt for TestReadWrite {
    fn characteristic(&self) -> esp_idf_svc::bt::ble::gatt::GattCharacteristic {
        GattCharacteristic::new(
            BtUuid::uuid16(0x1a19),
            enum_set!(Permission::Read),
            enum_set!(Property::Read),
            200,
            AutoResponse::ByApp
        )
    }
}

impl ReadExt for TestReadWrite {
    type State = ();
    fn on_read(&self, _state: Self::State) -> anyhow::Result<&[u8]> {
        Ok(&[2])
    }
}

#[derive(Debug, Clone, Default)]
struct TestReadWrite2;

impl CharacteristicExt for TestReadWrite2 {
    fn characteristic(&self) -> esp_idf_svc::bt::ble::gatt::GattCharacteristic {
        GattCharacteristic::new(
            BtUuid::uuid16(0xa223),
            enum_set!(Permission::Write),
            enum_set!(Property::Write),
            200,
            AutoResponse::ByApp
        )
    }
}

impl WriteExt for TestReadWrite2 {
    type State = ();
    fn on_write(&self, _state: Self::State, data: &[u8]) -> anyhow::Result<()> {
        log::warn!("write: {:?}", data);
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let (_, peripherals, nvs) = init()?;
    let driver = BtDriver::<Ble>::new(peripherals.modem, Some(nvs))?;
    let mut ble_app = ble::BLEApp
        ::builder()
        .device_name("esp32c3")
        .app_id(0)
        .driver(driver)?
        .state(())
        .adv_configuration(AdvConfiguration {
            include_name: true,
            include_txpower: true,
            flag: 2,
            ..Default::default()
        })
        .build();
    let mut service = Service::new(
        GattServiceId {
            id: GattId {
                uuid: BtUuid::uuid16(0xff32),
                inst_id: 0,
            },
            is_primary: true,
        },
        8
    );
    service.add_read_characteristic(Arc::new(TestReadWrite));
    service.add_write_characteristic(Arc::new(TestReadWrite2));
    ble_app.add_service(service);
    log::info!("start ble {:#?}", ble_app.services);
    ble::start(ble_app)?;

    loop {
        FreeRtos::delay_ms(1000);
    }
}
