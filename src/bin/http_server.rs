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
        nvs
    )?;
    let mut server = esp_idf_svc::http::server::EspHttpServer::new(
        &(Configuration {
            stack_size: 10240,
            ..Default::default()
        })
    )?;
    let led = peripherals.pins.gpio8;
    let channel = peripherals.rmt.channel0;
    let ws2812_rmt = Arc::new(Mutex::new(WS2812RMT::new(led, channel)?));
    let ws2812_rmt_clone = ws2812_rmt.clone();

    server.fn_handler("/set-color", Method::Post, move |mut req| {
        let params: Params = get_json_body(&mut req)?;
        let color = params.color;
        ws2812_rmt_clone.lock().unwrap().set_pixel(RGB8::new(color.r, color.g, color.b))?;
        log::info!("color: {:?}", color);

        let mut response = req.into_ok_response()?;
        response.write_all(b"OK")?;
        Ok::<(), anyhow::Error>(())
    })?;

    server.fn_handler("/shutdown", Method::Get, move |req| {
        ws2812_rmt.lock().unwrap().shutdown()?;
        let mut response = req.into_ok_response()?;
        response.write_all(b"OK")?;
        Ok::<(), anyhow::Error>(())
    })?;

    // 返回成功结果结束程序。
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    color: Color,
}

impl From<Color> for RGB8 {
    fn from(color: Color) -> Self {
        RGB8::new(color.r, color.g, color.b)
    }
}

fn get_json_body<'r, T: for<'b> Deserialize<'b>>(
    req: &mut Request<&mut EspHttpConnection<'r>>
) -> anyhow::Result<T> {
    let len = req.content_len().ok_or(anyhow!("content_len is None"))? as usize;
    let mut buf = vec![0;len];
    req.read_exact(&mut buf)?;
    Ok(serde_json::from_slice(&buf)?)
}
