use std::sync::{ Arc, Mutex };

use anyhow::anyhow;
use embedded_svc::http::Headers;
use esp_idf_svc::{
    http::{ server::{ Configuration, EspHttpConnection, Request }, Method },
    io::{ Read, Write },
};
use rgb::RGB8;
use rust_embedded_study::led::WS2812RMT;
use serde::{ Deserialize, Serialize };

// 配置结构体，包含WiFi的SSID和PSK
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> anyhow::Result<()> {
    // 初始化系统服务、外设和NVS
    // 初始化系统循环、外设和NVS闪存。
    let (sysloop, peripherals, nvs) = rust_embedded_study::init()?;

    // 连接到WiFi网络
    // 使用配置文件中的WiFi SSID和PSK连接到WiFi。
    let _wifi = rust_embedded_study::wifi::connect_wifi(
        CONFIG.wifi_ssid,
        &CONFIG.wifi_psk,
        peripherals.modem,
        sysloop,
        nvs
    )?;

    // 初始化HTTP服务器
    let mut server = esp_idf_svc::http::server::EspHttpServer::new(
        &(Configuration {
            stack_size: 10240,

            ..Default::default()
        })
    )?;

    // 初始化LED控制器
    let led = peripherals.pins.gpio8;
    let channel = peripherals.rmt.channel0;
    let ws2812_rmt = Arc::new(Mutex::new(WS2812RMT::new(led, channel)?));
    let ws2812_rmt_clone = ws2812_rmt.clone();

    server.fn_handler("/", Method::Get, |req| {
        let mut response = req.into_ok_response()?;
        response.write_all(include_bytes!("index.html"))?;
        Ok::<(), anyhow::Error>(())
    })?;

    // 注册处理设置LED颜色的HTTP请求的函数
    server.fn_handler("/set-color", Method::Post, move |mut req| {
        // 从请求中获取JSON数据并解析为颜色参数
        let params: Params = get_json_body(&mut req)?;
        let color = params.color;
        // 设置LED颜色
        ws2812_rmt_clone.lock().unwrap().set_pixel(RGB8::new(color.r, color.g, color.b))?;
        log::info!("color: {:?}", color);

        // 构建并返回成功的HTTP响应
        let mut response = req.into_response(
            200,
            Some("ok"),
            &[
                ("Access-Control-Allow-Origin", "*"),
                ("Access-Control-Allow-Methods", "all"),
            ]
        )?;
        response.write_all(b"OK")?;
        Ok::<(), anyhow::Error>(())
    })?;

    // 注册处理关闭LED的HTTP请求的函数
    server.fn_handler("/shutdown", Method::Get, move |req| {
        // 关闭LED控制器
        ws2812_rmt.lock().unwrap().shutdown()?;
        let mut response = req.into_response(
            200,
            Some("ok"),
            &[
                ("Access-Control-Allow-Origin", "*"),
                ("Access-Control-Allow-Methods", "all"),
            ]
        )?;
        response.write_all(b"OK")?;
        Ok::<(), anyhow::Error>(())
    })?;

    // 保持程序运行

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// 表示RGB颜色的结构体
#[derive(Debug, Serialize, Deserialize)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

// 包含颜色参数的结构体
#[derive(Debug, Serialize, Deserialize)]
struct Params {
    color: Color,
}

// 从Color结构体到RGB8的转换
impl From<Color> for RGB8 {
    fn from(color: Color) -> Self {
        RGB8::new(color.r, color.g, color.b)
    }
}

// 从HTTP请求中获取JSON负载的函数
fn get_json_body<'r, T: for<'b> Deserialize<'b>>(
    req: &mut Request<&mut EspHttpConnection<'r>>
) -> anyhow::Result<T> {
    // 获取请求的负载长度
    let len = req.content_len().ok_or(anyhow!("content_len is None"))? as usize;
    // 创建一个缓冲区用于存储负载
    let mut buf = vec![0; len];
    // 从请求中读取负载到缓冲区
    req.read_exact(&mut buf)?;
    // 从缓冲区中的数据解析JSON负载
    Ok(serde_json::from_slice(&buf)?)
}
