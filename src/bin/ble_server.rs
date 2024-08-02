use std::sync::{ Arc, Mutex };
use rgb::RGB8;
use rust_embedded_study::{ init, led::WS2812RMT };
use esp32_nimble::{ utilities::BleUuid, BLEAdvertisementData, BLEDevice, NimbleProperties };

fn main() -> anyhow::Result<()> {
    // 初始化系统、外设和NVS flash。
    let (_sys, peripherals, _nvs) = init()?;

    // 获取BLE设备实例
    let device = BLEDevice::take();

    // 初始化LED灯
    let led = Arc::new(
        Mutex::new(WS2812RMT::new(peripherals.pins.gpio8, peripherals.rmt.channel0)?)
    );

    // 获取并配置BLE的广告实例
    let advertising = device.get_advertising();

    // 获取并配置BLE的服务实例。
    let server = device.get_server();

    // 配置BLE连接时的回调函数
    server.on_connect(|server, desc| {
        log::info!("on_connect: {:#?}", desc);
        server.update_conn_params(desc.conn_handle(), 24, 48, 0, 60).unwrap();
        if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
            advertising.lock().start().unwrap();
        }
    });

    // 配置BLE断开连接时的回调函数
    server.on_disconnect(|desc, reason| {
        log::warn!("on_disconnect: {:#?}, reason: {:#?}", desc, reason)
    });

    // 创建BLE服务，使用UUID 0x8848
    let service = server.create_service(BleUuid::from_uuid16(0x8848));

    // 在服务中创建一个特性，用于设置LED颜色，使用UUID 0xffa1
    let set_color_characteristic = service
        .lock()
        .create_characteristic(
            BleUuid::from_uuid16(0xffa1),
            NimbleProperties::WRITE | NimbleProperties::READ
        );

    // 在服务中创建一个特性，用于关闭LED，使用UUID 0xffa2
    let close_characteristic = service
        .lock()
        .create_characteristic(BleUuid::from_uuid16(0xffa2), NimbleProperties::WRITE);

    // 当设置颜色特性被写入时，更新LED的颜色。
    let write_led = led.clone();
    set_color_characteristic
        .lock()
        .on_write(move |args| {
            let data = args.recv_data();
            match write_led.lock().unwrap().set_pixel(RGB8::new(data[0], data[1], data[2])) {
                Ok(_) => {
                    log::warn!("Set LED color to {:?}", RGB8::new(data[0], data[1], data[2]))
                }
                Err(e) => log::error!("Error: {}", e),
            }
        })
        .on_read(|value, desc| {
            log::warn!("Read from descriptor {:?}", desc);
            value.set_value(b"hello world")
        });

    // 当关闭特性被写入时，关闭LED。
    close_characteristic.lock().on_write(move |args| {
        let data = args.recv_data();
        if data[0] == 1 {
            match led.lock().unwrap().shutdown() {
                Ok(_) => { log::warn!("Close LED {:?}", data) }
                Err(e) => log::error!("Error: {}", e),
            }
        }
    });

    // 配置广告数据并启动广告
    advertising
        .lock()
        .set_data(
            BLEAdvertisementData::new().name("ESP32").add_service_uuid(BleUuid::from_uuid16(0x8848))
        )?;
    advertising.lock().start()?;
    // 打印蓝牙服务相关日志
    server.ble_gatts_show_local();

    Ok(())
}
