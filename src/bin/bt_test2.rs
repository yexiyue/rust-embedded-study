use std::sync::Arc;

use enumset::enum_set;
use esp_idf_svc::{
    bt::{
        ble::{
            gap::{ AdvConfiguration, SecurityConfiguration },
            gatt::{
                AutoResponse,
                GattCharacteristic,
                GattId,
                GattResponse,
                GattServiceId,
                GattStatus,
                Permission,
                Property,
            },
        },
        Ble,
        BtDriver,
        BtStatus,
        BtUuid,
    },
    hal::delay::FreeRtos,
};
use rust_embedded_study::init;
use esp_idf_svc::bt::ble::{ gap, gatt };
use log::{ info, error, warn };
fn main() -> anyhow::Result<()> {
    let (_, peripherals, nvs) = init()?;
    let driver = Arc::new(BtDriver::<Ble>::new(peripherals.modem, Some(nvs))?);

    let gap = Arc::new(gap::EspBleGap::new(driver.clone())?);

    // gap.start_advertising()?;

    let gap_clone = gap.clone();

    gap_clone.subscribe(move |event| {
        info!("gap event: {:#?}", event);
        if let gap::BleGapEvent::AdvertisingConfigured(status) = event {
            check_gap_status(status);
            gap.start_advertising().unwrap();
        }
        if let gap::BleGapEvent::ChannelsConfigured(status) = event {
            check_gap_status(status);
            gap.stop_advertising().unwrap()
        }
    })?;
    gap_clone.set_device_name("ESP32")?;
    // 调用这个会触发AdvertisingConfigured
    gap_clone.set_adv_conf(
        &(AdvConfiguration {
            include_name: true,
            include_txpower: true,
            flag: 2,
            ..Default::default()
        })
    )?;

    let gatt = Arc::new(gatt::server::EspGatts::new(driver)?);
    let gatt_clone = gatt.clone();
    // let gap2 = gap_clone.clone();
    gatt_clone.subscribe(move |(gatt_if, event)| {
        info!("gatt event: {:#?} gatt_if:{gatt_if}", event);
        match event {
            gatt::server::GattsEvent::ServiceRegistered { .. } => {
                gatt.create_service(
                    gatt_if,
                    &(GattServiceId {
                        id: GattId {
                            uuid: BtUuid::uuid16(0xff32),
                            inst_id: 0,
                        },
                        is_primary: true,
                    }),
                    8
                ).unwrap();
            }
            gatt::server::GattsEvent::ServiceCreated { status, service_handle, service_id } => {
                info!("Service created: {:?}", service_id);
                gatt.start_service(service_handle).unwrap();
                gatt.add_characteristic(
                    service_handle,
                    &(GattCharacteristic {
                        uuid: BtUuid::uuid16(0xaa12),
                        permissions: enum_set!(Permission::Write | Permission::Read),
                        properties: enum_set!(Property::Write | Property::Read),
                        max_len: 400,
                        auto_rsp: AutoResponse::ByApp,
                    }),
                    &[]
                ).unwrap();

                gatt.add_characteristic(
                    service_handle,
                    &(GattCharacteristic {
                        uuid: BtUuid::uuid16(0xaa23),
                        permissions: enum_set!(Permission::Read),
                        properties: enum_set!(Property::Indicate),
                        max_len: 400,
                        auto_rsp: AutoResponse::ByApp,
                    }),
                    &[1, 2]
                ).unwrap();

                // gatt.add_characteristic(
                //     service_handle,
                //     &(GattCharacteristic {
                //         uuid: BtUuid::uuid16(0xaa45),
                //         permissions: enum_set!(Permission::Read),
                //         properties: enum_set!(Property::Notify),
                //         max_len: 400,
                //         auto_rsp: AutoResponse::ByApp,
                //     }),
                //     &[1, 2]
                // ).unwrap();

                gatt.add_characteristic(
                    service_handle,
                    &(GattCharacteristic {
                        uuid: BtUuid::uuid16(0xaa66),
                        permissions: enum_set!(Permission::Read),
                        properties: enum_set!(Property::Read),
                        max_len: 400,
                        auto_rsp: AutoResponse::ByApp,
                    }),
                    &[2, 1]
                ).unwrap();
            }
            // gatt::server::GattsEvent::Read {
            //     conn_id,
            //     trans_id,
            //     addr,
            //     handle,
            //     offset,
            //     is_long,
            //     need_rsp,
            // } => {
            //     log::info!("{addr:?} read  conn_id:{:?} value:{:?}", conn_id, trans_id);
            //     if need_rsp {
            //         let mut response = GattResponse::new();
            //         response.attr_handle(handle).auth_req(0).offset(offset).value(&[1, 2]).unwrap();
            //         gatt.send_response(
            //             gatt_if,
            //             conn_id,
            //             trans_id,
            //             GattStatus::Ok,
            //             Some(&response)
            //         ).unwrap();
            //     }
            // }
            gatt::server::GattsEvent::Write {
                conn_id,
                trans_id,
                addr,
                handle,
                offset,
                need_rsp,
                is_prep,
                value,
            } => {
                log::info!("{addr:?} write  conn_id:{:?} value:{:?}", conn_id, value);
                if need_rsp {
                    if is_prep {
                        let mut response = GattResponse::new();
                        response
                            .attr_handle(handle)
                            .auth_req(0)
                            .offset(offset)
                            .value(value)
                            .unwrap();

                        gatt.send_response(
                            gatt_if,
                            conn_id,
                            trans_id,
                            GattStatus::Ok,
                            Some(&response)
                        ).unwrap();
                    } else {
                        gatt.send_response(
                            gatt_if,
                            conn_id,
                            trans_id,
                            GattStatus::Ok,
                            None
                        ).unwrap();
                    }
                }
            }
            _ => {}
        }
    })?;
    gatt_clone.register_app(0)?;
    gatt_clone.register_app(1)?;

    loop {
        FreeRtos::delay_ms(1000);
    }
}

fn check_gatt_status(status: GattStatus) {
    if !matches!(status, GattStatus::Ok) {
        error!("Gatt status: {:?}", status)
    }
}

fn check_gap_status(status: BtStatus) {
    if !matches!(status, BtStatus::Success) {
        error!("Bt status: {:?}", status)
    }
}
