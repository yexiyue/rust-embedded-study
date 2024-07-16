use std::time::Duration;

use anyhow::Result;
use esp_idf_svc::hal::{
    gpio::OutputPin,
    peripheral::Peripheral,
    rmt::{ config::TransmitConfig, FixedLengthSignal, PinState, Pulse, RmtChannel, TxRmtDriver },
};

pub use rgb::RGB8;


pub struct WS2812RMT<'a> {
    tx_rmt_derive: TxRmtDriver<'a>,
}

impl<'a> WS2812RMT<'a> {
    pub fn new(
        led: impl Peripheral<P = impl OutputPin> + 'a,
        channel: impl Peripheral<P = impl RmtChannel> + 'a
    ) -> Result<Self> {
        // 配置RMT的传输参数
        let config = TransmitConfig::new().clock_divider(2);
        // 初始化RMT驱动
        let tx = TxRmtDriver::new(channel, led, &config)?;
        Ok(Self { tx_rmt_derive: tx })
    }

    pub fn set_pixel(&mut self, rgb: RGB8) -> Result<()> {
        // 将RGB颜色值转换为一个32位的整数。
        // RGB颜色由红、绿、蓝三部分组成，每部分占用8位。
        // 这里通过位移操作将它们组合在一起。
        let color: u32 = ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | (rgb.b as u32);

        // 获取发送器的时钟频率，这将用于计算脉冲的持续时间。
        let ticks_hz = self.tx_rmt_derive.counter_clock()?;

        // 定义一个短的高电平脉冲，通常用于表示二进制中的'0'
        let t0h = Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(400))?;
        // 定义一个短的低电平脉冲，与上面的高电平脉冲一起构成一个完整的'0'脉冲对
        let t0l = Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(850))?;

        // 定义一个长的高电平脉冲，通常用于表示二进制中的'1'
        let t1h = Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(800))?;
        // 定义一个长的低电平脉冲，与上面的高电平脉冲一起构成一个完整的'1'脉冲对
        let t1l = Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(450))?;

        // 创建一个固定长度为24的信号序列，用于存储脉冲对。
        let mut signal = FixedLengthSignal::<24>::new();
        
        // 生成RMT脉冲序列来表示颜色
        // 从最高位开始遍历颜色值的每一位（从23到0）
        for i in (0..24).rev() {
            // 计算当前位的权重，即2的i次方
            let p = (2u32).pow(i);

            // 检查当前位是否为1，如果为1则bit为true，否则为false
            let bit = (p & color) != 0;

            // 根据bit的值选择脉冲对：如果bit为true则选择表示'1'的脉冲对，否则选择表示'0'的脉冲对
            let pulse = if bit { (t1h, t1l) } else { (t0h, t0l) };

            // 将选择的脉冲对设置到信号序列中对应的位置上
            // 注意，由于是从高位到低位遍历，所以位置需要从23开始递减
            signal.set(23 - (i as usize), &pulse)?;
        }
        Ok(self.tx_rmt_derive.start_blocking(&signal)?)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.set_pixel(RGB8::new(0, 0, 0))?;
        Ok(())
    }
}

// // 调整颜色亮度
// pub fn adjust_brightness(rgb: RGB8, brightness: f32) -> RGB8 {
//     let factor = brightness.max(0.0).min(1.0); // 确保亮度因子在有效范围内

//     // 调整每个颜色分量
//     let new_r = (rgb.r as f32) * factor;
//     let new_g = (rgb.g as f32) * factor;
//     let new_b = (rgb.b as f32) * factor;

//     // 将结果转换回u8类型，同时确保不会溢出
//     let new_r = new_r.min(255.0).max(0.0) as u8;
//     let new_g = new_g.min(255.0).max(0.0) as u8;
//     let new_b = new_b.min(255.0).max(0.0) as u8;

//     RGB8::new(new_r, new_g, new_b)
// }

// // sin周期变化
// pub fn cycle_value_sin(t: f32) -> f32 {
//     ((t * std::f32::consts::PI).sin() + 1.0) / 2.0
// }

// // 线性周期变化
// pub fn cycle_value<'a>(value: &'a mut f32, step: f32) -> impl (FnMut() -> f32) + 'a {
//     let mut operator = 1.0;
//     move || {
//         if *value >= 1.0 {
//             operator = -1.0;
//         }
//         if *value <= 0.0 {
//             operator = 1.0;
//         }
//         *value += operator * step;
//         *value
//     }
// }
