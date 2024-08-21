// 导入标准库和相关外部 crate，用于硬件抽象、多线程通信等
use std::{
    num::NonZeroU32,
    sync::{mpsc::channel, Arc},
};

// 导入 esp-idf-svc 和 hal 层相关模块，用于 GPIO 操作和任务通知
use esp_idf_svc::hal::{
    gpio::{InterruptType, PinDriver, Pull},
    task::notification::Notification,
};
// 导入项目中用于控制 WS2812 LED 的模块
use rust_embedded_study::led::WS2812RMT;

// 主函数，返回一个 Result 类型以处理可能的错误
fn main() -> anyhow::Result<()> {
    // 初始化系统和外设
    let (_sys, peripherals, _nvs) = rust_embedded_study::init()?;

    // 创建 WS2812 LED 驱动实例
    let mut led = WS2812RMT::new(peripherals.pins.gpio8, peripherals.rmt.channel0)?;
    // 创建按钮输入引脚驱动实例
    let mut button = PinDriver::input(peripherals.pins.gpio9)?;

    // 创建一个无缓冲的通道，用于在线程间传输数据
    let (tx, rx) = channel::<bool>();
    // 配置按钮引脚为上拉，并设置中断类型为上升沿触发
    button.set_pull(Pull::Up)?;
    button.set_interrupt_type(InterruptType::PosEdge)?;

    // 创建一个新的线程来处理按钮中断
    std::thread::spawn(move || -> Result<(), anyhow::Error> {
        // 创建一个通知对象，用于在中断发生时通知其他线程
        let notification = Arc::new(Notification::new());
        let notifier = notification.notifier();
        // 定义一个变量来表示 LED 的状态（开/关）
        let mut open = false;
        // 使用 unsafe 块来调用可能不安全的中断订阅函数
        unsafe {
            button.subscribe(move || {
                notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
            })?;
        }
        // 无限循环，等待中断发生并切换 LED 状态
        loop {
            button.enable_interrupt()?;
            notification.wait(esp_idf_svc::hal::delay::BLOCK);
            open = !open;
            tx.send(open)?;
        }
    });

    // 主线程循环接收 LED 状态，并根据状态设置 LED 颜色
    while let Ok(open) = rx.recv() {
        if open {
            led.set_pixel(rust_embedded_study::led::RGB8::new(255, 255, 0))?;
        } else {
            led.set_pixel(rust_embedded_study::led::RGB8::new(0, 0, 0))?;
        }
    }

    Ok(())
}
