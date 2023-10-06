#![no_std]
#![no_main]
use adafruit_seesaw::{devices::GenericDevice, prelude::*, Seesaw};
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    gpio::GpioExt,
    i2c::I2c,
    pac,
    prelude::*,
    rcc::{RccExt, SYSCLK_MAX},
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Starting");
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let gpiob = dp.GPIOB.split();
    let clocks = dp.RCC.constrain().cfgr.sysclk(SYSCLK_MAX.Hz()).freeze();
    let delay = cp.SYST.delay(&clocks);
    let scl = gpiob.pb6.into_alternate_open_drain::<4>();
    let sda = gpiob.pb7.into_alternate_open_drain::<4>();
    let mut i2c = I2c::new(dp.I2C1, (scl, sda), 400.kHz(), &clocks);
    // NOTE: use external library (e.g. embedded-hal-bus) to create I2c bus
    //
    // let bus = shared_bus::BusManagerSimple::new(i2c);
    let mut seesaw = Seesaw::new(delay);
    let device = GenericDevice::new_with_default_addr();
    let mut device = device.with_driver(seesaw.borrow_i2c(&mut i2c));
    device.init().expect("Failed to init generic device");

    let id = device.hardware_id().expect("Failed to get hardware id");
    rprintln!("Hardware ID {:?}", id);
    rprintln!(
        "Capabilities {:#?}",
        device.capabilities().expect("Failed to get options")
    );
    rprintln!(
        "Product info {:#?}",
        device.product_info().expect("failed to get product info")
    );
    #[allow(clippy::empty_loop)]
    loop {}
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
