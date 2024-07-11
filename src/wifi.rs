use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};

/**
 * 连接到指定的Wi-Fi网络。
 * 
 * 此函数初始化Wi-Fi模块，配置连接参数，并尝试连接到指定的Wi-Fi网络。它还负责打印扫描到的Wi-Fi网络信息，
 * 以及在成功连接后打印设备的IP地址信息。
 * 
 * @param ssid Wi-Fi网络的SSID。
 * @param psk Wi-Fi网络的预共享密钥（PSK）。
 * @param modem 用于与Wi-Fi模块通信的外设接口。
 * @param sysloop 系统事件循环，用于处理Wi-Fi相关的事件。
 * @param nvs NVS（Non-Volatile Storage）分区，用于存储Wi-Fi配置等信息。
 * @return 返回一个封装了Wi-Fi模块的Box<EspWifi>实例，表示连接成功；如果连接失败，则返回错误。
 */
pub fn connect_wifi(
    ssid: &str,
    psk: &str,
    modem: impl Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> Result<Box<EspWifi<'static>>> {
    // 初始化EspWifi实例。
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs))?;
    // 将EspWifi封装为BlockingWifi，以便可以使用阻塞模式的API。
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    // 配置Wi-Fi连接参数，包括SSID、认证方法和密码。
    let configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        password: psk.try_into().unwrap(),
        ..Default::default()
    });
    // 应用配置。
    wifi.set_configuration(&configuration)?;
    // 启动Wi-Fi模块。
    log::info!("启动Wi-Fi");
    wifi.start()?;
    // 扫描可用的Wi-Fi网络。
    log::info!("扫描Wi-Fi");
    let access_point_infos = wifi.scan()?;
    // 打印扫描结果。
    log::info!("扫描到的Wi-Fi数量: {}", access_point_infos.len());
    access_point_infos
        .into_iter()
        .for_each(|info| println!("{:#?}", info));

    // 尝试连接到配置的Wi-Fi网络。
    log::info!("连接Wi-Fi");
    wifi.connect()?;
    // 确认Wi-Fi连接已建立。
    log::info!("Wi-Fi已连接");
    // 等待网络接口启动。
    wifi.wait_netif_up()?;

    // 获取连接后的IP地址信息。
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    // 打印IP地址信息。
    log::info!("IP信息: {:?}", ip_info);

    // 返回封装了Wi-Fi模块的实例。
    Ok(Box::new(esp_wifi))
}