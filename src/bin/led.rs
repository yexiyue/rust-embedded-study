use std::time::{ Duration, Instant };
use esp_idf_svc::hal::peripherals::Peripherals;
use rust_embedded_study::led::{ RGB8, WS2812RMT };

fn main() -> anyhow::Result<()> {
    // 链接ESP-IDF的补丁，以确保程序与ESP-IDF环境正确集成
    esp_idf_svc::sys::link_patches();
    // 初始化日志系统，以便能够输出日志信息
    esp_idf_svc::log::EspLogger::initialize_default();
    // 获取板子的外设，用于后续控制LED
    let peripherals = Peripherals::take()?;
    // 选取GPIO8作为LED的控制引脚
    let led = peripherals.pins.gpio8;
    // 选取RMT的通道0作为WS2812的通信通道
    let channel = peripherals.rmt.channel0;
    // 初始化WS2812LED控制器
    let mut ws2812 = WS2812RMT::new(led, channel)?;

    // 记录程序启动的瞬间，用于计算运行时间
    let instant = Instant::now();

    // 无限循环，用于控制LED的点亮和关闭
    loop {
        // 检查是否已经运行了3秒以上
        if instant.elapsed() > Duration::from_secs(3) {
            // 如果超过3秒，关闭LED，并记录日志
            log::info!("关灯");
            ws2812.shutdown()?;
            std::thread::sleep(Duration::from_secs(1));
            log::info!("关机");
            return Ok(());
        } else {
            // 如果未超过3秒，点亮LED为绿色，并记录当前运行时间
            ws2812.set_pixel(RGB8::new(0, 255, 0))?;
            log::info!("time:{}", instant.elapsed().as_secs());
        }
        // 每次循环等待1秒，以便观察LED的状态变化
        std::thread::sleep(Duration::from_secs(1));
    }
}
