#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> anyhow::Result<()> {
    // 初始化系统循环、外设和NVS闪存。
    let (sysloop, peripherals, nvs) = rust_embedded_study::init()?;

    // 使用配置文件中的WiFi SSID和PSK连接到WiFi。
    // 这里使用了`rust_embedded_study`库提供的`connect_wifi`函数。
    let _wifi = rust_embedded_study::wifi::connect_wifi(
        CONFIG.wifi_ssid,
        &CONFIG.wifi_psk,
        peripherals.modem,
        sysloop,
        nvs,
    )?;
    let _server = rust_embedded_study::server::create_server()?;
    // 返回成功结果结束程序。
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
