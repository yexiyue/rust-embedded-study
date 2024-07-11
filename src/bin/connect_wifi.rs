#[toml_cfg::toml_config]
#[derive(Debug)]
/// `Config`结构体用于存储配置信息，主要是WiFi的SSID和PSK。
/// 
/// # Attributes
/// 
/// `wifi_ssid` - WiFi的SSID，默认值为"Wokwi-GUEST"。
/// `wifi_psk` - WiFi的预共享密钥（PSK），默认为空字符串。
pub struct Config {
    #[default("Wokwi-GUEST")]
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
    
    // 返回成功结果结束程序。
    Ok(())
}