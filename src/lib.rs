#![no_std]
#![allow(const_evaluatable_unchecked, incomplete_features)]
#![feature(array_try_map, generic_const_exprs)]
#![feature(async_fn_in_trait)]
// TODO improve the organization of the exports/visibility
use embedded_hal::delay::DelayUs;
mod common;
pub mod devices;
mod driver;
mod macros;
pub mod modules;
pub use common::*;
pub use devices::*;
pub use driver::*;

pub mod prelude {
    pub use super::{
        devices::*,
        driver::DriverExt,
        modules::{adc::*, encoder::*, gpio::*, neopixel::*, status::*, timer::*},
        SeesawDevice, SeesawDeviceInit,
    };
}

pub struct Seesaw<DELAY, I2C> {
    delay: DELAY,
    i2c: I2C,
}

impl<DELAY, I2C> Seesaw<DELAY, I2C>
where
    DELAY: DelayUs,
    I2C: I2cDriver,
{
    pub fn new(delay: DELAY, i2c: I2C) -> Self {
        Seesaw { delay, i2c }
    }
}

impl<DELAY, I2C> Driver for Seesaw<DELAY, I2C>
where
    DELAY: DelayUs,
    I2C: I2cDriver,
{
    type Delay = DELAY;
    type I2c = I2C;
    type I2cError = I2C::Error;

    fn delay(&mut self) -> &mut Self::Delay {
        &mut self.delay
    }

    fn i2c(&mut self) -> &mut Self::I2c {
        &mut self.i2c
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SeesawError<E> {
    /// I2C bus error
    I2c(E),
    /// Occurs when an invalid hardware ID is read
    InvalidHardwareId(u8),
}
impl<E> From<E> for SeesawError<E> {
    fn from(value: E) -> Self {
        Self::I2c(value)
    }
}

pub trait SeesawDevice {
    type Error;
    type Driver: Driver;

    const DEFAULT_ADDR: u8;
    const HARDWARE_ID: HardwareId;
    const PRODUCT_ID: u16;

    fn addr(&self) -> u8;

    fn driver(&mut self) -> &mut Self::Driver;

    fn new(addr: u8, driver: Self::Driver) -> Self;

    fn new_with_default_addr(driver: Self::Driver) -> Self;
}

/// At startup, Seesaw devices typically have a unique set of initialization
/// calls to be made. e.g. for a Neokey1x4, we're need to enable the on-board
/// neopixel and also do some pin mode setting to get everything working.
/// All devices implement `DeviceInit` with a set of sensible defaults. You can
/// override the default initialization function with your own by calling
/// `Seesaw::connect_with` instead of `Seesaw::connect`.
pub trait SeesawDeviceInit<D: Driver>: SeesawDevice<Driver = D>
where
    Self: Sized,
{
    fn init(self) -> Result<Self, Self::Error>;
}
