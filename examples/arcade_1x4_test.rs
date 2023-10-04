#![no_std]
#![no_main]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
use adafruit_seesaw::{devices::ArcadeButton1x4, prelude::*, Seesaw};
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{gpio::GpioExt, i2c::I2c, pac, prelude::*, rcc::RccExt};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let gpiob = dp.GPIOB.split();
    let clocks = dp.RCC.constrain().cfgr.freeze();
    let delay = cp.SYST.delay(&clocks);
    let scl = gpiob.pb6.into_alternate_open_drain::<4>();
    let sda = gpiob.pb7.into_alternate_open_drain::<4>();
    let i2c = I2c::new(dp.I2C1, (scl, sda), 100.kHz(), &clocks);
    let seesaw = Seesaw::new(delay, i2c);
    let mut arcade = ArcadeButton1x4::new_with_default_addr(seesaw)
        .init()
        .expect("Failed to start ArcadeButton1x4");

    loop {
        let buttons = arcade.button_values().expect("Failed to get button values");
        arcade
            .set_led_duty_cycles(&buttons.map(|on| if on { 0xFFu8 } else { 0x1F }))
            .expect("Failed to set LED duty cycles");
    }
}

#[panic_handler]
fn handle_panic(info: &core::panic::PanicInfo) -> ! {
    rprintln!("PANIC! {}", info);
    rprintln!("Location {:?}", info.location());
    if let Some(pl) = info.payload().downcast_ref::<&str>() {
        rprintln!("Payload {:?}", pl);
    }
    loop {}
}
