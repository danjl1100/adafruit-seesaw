#![no_std]
#![allow(const_evaluatable_unchecked, incomplete_features)]
#![feature(array_try_map, generic_const_exprs)]
#![feature(async_fn_in_trait)]

use embedded_hal::delay::DelayUs;

// TODO improve the organization of the exports/visibility
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

pub struct Seesaw<DELAY> {
    delay: DELAY,
}
pub struct SeesawBorrowed<'a, DELAY, I2C> {
    delay: &'a mut DELAY,
    i2c: I2C,
}

impl<DELAY> Seesaw<DELAY> {
    pub fn new(delay: DELAY) -> Self {
        Seesaw { delay }
    }

    pub fn borrow_i2c<I2C: I2cDriver>(&mut self, i2c: I2C) -> SeesawBorrowed<'_, DELAY, I2C> {
        let Self { delay, .. } = self;
        SeesawBorrowed { delay, i2c }
    }

    pub fn borrow_i2c_async<I2C: I2cDriverAsync>(
        &mut self,
        i2c: I2C,
    ) -> SeesawBorrowed<'_, DELAY, I2C> {
        let Self { delay, .. } = self;
        SeesawBorrowed { delay, i2c }
    }
}

impl<DELAY, I2C> Driver for SeesawBorrowed<'_, DELAY, I2C>
where
    DELAY: DelayUs,
    I2C: I2cDriver,
{
    type Delay = DELAY;
    type I2c = I2C;
    type I2cError = I2C::Error;

    fn delay(&mut self) -> &mut Self::Delay {
        self.delay
    }

    fn i2c(&mut self) -> &mut Self::I2c {
        &mut self.i2c
    }
}

impl<DELAY, I2C> DriverAsync for SeesawBorrowed<'_, DELAY, I2C>
where
    DELAY: DelayAsync,
    I2C: I2cDriverAsync,
{
    type Delay = DELAY;
    type I2c = I2C;
    type I2cError = I2C::Error;

    fn delay(&mut self) -> &mut Self::Delay {
        self.delay
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

macro_rules! seesaw_device_def {
    (
        for $async:ident;
        device { $device_name:ident < $driver:ident >  }
        device_init { $device_init_name:ident }
    ) => {
        pub trait $device_name {
            type Error;
            type Driver: $driver;

            const DEFAULT_ADDR: u8;
            const HARDWARE_ID: HardwareId;
            const PRODUCT_ID: u16;

            fn addr(&self) -> u8;

            fn driver(&mut self) -> &mut Self::Driver;
        }
        /// At startup, Seesaw devices typically have a unique set of initialization
        /// calls to be made. e.g. for a Neokey1x4, we're need to enable the on-board
        /// neopixel and also do some pin mode setting to get everything working.
        /// All devices implement `DeviceInit` with a set of sensible defaults. You can
        /// override the default initialization function with your own by calling
        /// `Seesaw::connect_with` instead of `Seesaw::connect`.
        pub trait $device_init_name<D: $driver>: $device_name<Driver = D>
        where
            Self: Sized,
        {
            maybe_async! {
                $async @fn init (&mut self) -> Result<(), Self::Error>;
            }
        }
    };
}
seesaw_device_def! {
    for blocking;
    device { SeesawDevice <Driver> }
    device_init { SeesawDeviceInit }
}
seesaw_device_def! {
    for async;
    device { SeesawDeviceAsync <DriverAsync> }
    device_init { SeesawDeviceInitAsync }
}
