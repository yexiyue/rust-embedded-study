use std::sync::{ mpsc::{ channel, Sender }, Arc, Condvar, Mutex };

use anyhow::anyhow;
use embedded_svc::http::{ client::Client, Headers };
use esp32_nimble::{ utilities::BleUuid, BLEAdvertisementData, NimbleProperties };
use esp_idf_svc::{
    http::{ client::{ Configuration, EspHttpConnection }, Method },
    ota::{ EspOta, FirmwareInfo },
};

// 配置结构体，用于读取配置文件
#[toml_cfg::toml_config]
#[derive(Debug)]
pub struct Config {
    // 默认SSID和密码
    #[default("Wokwi-GUEST")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

// 常量定义，用于固件下载的chunk大小、最大和最小尺寸
const FIRMWARE_DOWNLOAD_CHUNK_SIZE: usize = 1024 * 20;
const FIRMWARE_MAX_SIZE: usize = 1024 * 1024;
const FIRMWARE_MIN_SIZE: usize = size_of::<FirmwareInfo>() + 1024;

// 主函数，程序的入口点
fn main() -> anyhow::Result<()> {
    // 初始化系统循环、外设和NVS闪存
    let (sysloop, peripherals, nvs) = rust_embedded_study::init()?;

    // 连接WiFi
    let _wifi = rust_embedded_study::wifi::connect_wifi(
        CONFIG.wifi_ssid,
        &CONFIG.wifi_psk,
        peripherals.modem,
        sysloop,
        nvs
    )?;

    // 初始化BLE设备和广告
    let device = esp32_nimble::BLEDevice::take();
    let advertising = device.get_advertising();

    // 获取BLE服务器
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

    // 创建BLE服务和OTA特性
    let services = server.create_service(BleUuid::from_uuid16(0x8849));
    let ota_characteristic = services
        .lock()
        .create_characteristic(
            BleUuid::from_uuid16(0xffa1),
            NimbleProperties::WRITE | NimbleProperties::NOTIFY
        );
    // 创建消息通道
    let (tx, rx) = channel::<(usize, usize)>();

    // 创建共享状态
    let state = Arc::new((Mutex::new(false), Condvar::new()));

    // 克隆状态，用于OTA写入时的通知
    let state_clone = state.clone();
    // 配置OTA特性的写入回调
    ota_characteristic
        .lock()
        .on_write(move |args| {
            let data = args.recv_data();
            if data[0] == 1 {
                let mut is_start = state_clone.0.lock().unwrap();
                *is_start = true;
                state_clone.1.notify_all();
            }
            if data[0] == 2 {
                unsafe {
                    esp_idf_svc::sys::esp_restart();
                }
            }
        })
        .create_2904_descriptor();

    // 创建线程进行OTA更新
    std::thread::spawn(move || {
        let (lock, condvar) = &*state;
        let mut is_start = lock.lock().unwrap();
        while !*is_start {
            is_start = condvar.wait(is_start).unwrap();
        }
        log::warn!("Start OTA");
        firmware("http://192.168.88.235:5500/ble_server.bin", tx)
    });

    // 配置广告数据并启动广告
    advertising
        .lock()
        .set_data(
            BLEAdvertisementData::new()
                .name("ESP32_OTA")
                .add_service_uuid(BleUuid::from_uuid16(0x8849))
        )?;
    advertising.lock().start()?;
    // 打印蓝牙服务相关日志
    server.ble_gatts_show_local();

    // 接收并处理下载进度
    while let Ok((read_len, file_size)) = rx.recv() {
        let percent = (read_len * 100) / file_size;
        ota_characteristic
            .lock()
            .set_value(&[percent as u8])
            .notify();
    }

    Ok(()) 
}

// 固件下载和更新函数
fn firmware(uri: &str, tx: Sender<(usize, usize)>) -> anyhow::Result<()> {
    // 初始化HTTP客户端
    let mut client = Client::wrap(
        EspHttpConnection::new(
            &(Configuration {
                buffer_size: Some(1024 * 4),
                ..Default::default()
            })
        )?
    );
    // 准备下载请求
    let request = client.request(Method::Get, uri, &[("Accept", "application/octet-stream")])?;
    let mut response = request.submit()?;
    // 检查响应状态
    if response.status() != 200 {
        return Err(anyhow!("Firmware download failed: {}", response.status()));
    }
    // 获取文件大小并进行校验
    let file_size = response.content_len().unwrap_or(0) as usize;
    if file_size <= FIRMWARE_MIN_SIZE {
        return Err(
            anyhow!(
                "File size is {file_size}, too small to be a firmware! No need to proceed further."
            )
        );
    }
    if file_size > FIRMWARE_MAX_SIZE {
        return Err(anyhow!("File is too big ({file_size} bytes)."));
    }
    log::warn!("file size: {file_size}");
    // 初始化OTA更新
    let mut ota = EspOta::new()?;
    let mut buff = vec![0;FIRMWARE_DOWNLOAD_CHUNK_SIZE];
    let mut total_read_len = 0usize;
    // 开始OTA更新
    let mut work = ota.initiate_update()?;
    // 循环读取和写入数据
    let dl_result = loop {
        let n = response.read(&mut buff)?;
        total_read_len += n;
        tx.send((total_read_len, file_size))?;

        if n > 0 {
            if let Err(e) = work.write(&buff[..n]) {
                log::error!("Failed to write to OTA. {e}");
                break Err(anyhow!(e));
            }
        }
        if total_read_len >= file_size {
            break Ok(());
        }
    };
    // 检查下载结果并相应处理
    if dl_result.is_err() {
        return Ok(work.abort()?);
    }
    if total_read_len < file_size {
        log::error!(
            "Supposed to download {file_size} bytes, but we could only get {total_read_len}. May be network error?"
        );
        return Ok(work.abort()?);
    }
    // 完成OTA更新
    work.complete()?;
    log::info!("OTA done!");
    Ok(())
}