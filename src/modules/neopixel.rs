use crate::{
    common::{Modules, Reg},
    maybe_async, DelayAsync, Driver, DriverAsync, DriverExt, DriverExtAsync, SeesawDevice,
    SeesawDeviceAsync, SeesawError,
};
use embedded_hal::delay::DelayUs;

/// WO - 8 bits
/// This register sets the pin number (PORTA) that is used for the NeoPixel
/// output.
const SET_PIN: &Reg = &[Modules::Neopixel.into_u8(), 0x01];
/// WO - 8 bits
/// The protocol speed. (see `NeopixelSpeed`) Default is 800khz.
const SET_SPEED: &Reg = &[Modules::Neopixel.into_u8(), 0x02];
/// WO - 16 bits
/// The number of bytes currently used for the pixel array. This is
/// dependent on when the pixels you are using are RGB or RGBW.
const SET_LEN: &Reg = &[Modules::Neopixel.into_u8(), 0x03];
/// WO - 256 bits (32 bytes)
/// The data buffer. The first 2 bytes are the start address, and the data
/// to write follows. Data should be written in blocks of maximum size 30
/// bytes at a time.
const SET_BUF: &Reg = &[Modules::Neopixel.into_u8(), 0x04];
/// W0 - Zero bits
/// Sending the SHOW command will cause the output to update. There's no
/// arguments/data after the command.
const SHOW: &Reg = &[Modules::Neopixel.into_u8(), 0x05];

macro_rules! neopixel_def {
    (
        for $async:ident;
        neopixel { $name:ident < $driver:ident > }
        seesaw_device { $seesaw_device:ident }
    ) => {
        pub trait $name<D: $driver>: $seesaw_device<Driver = D> {
            const PIN: u8;

            /// The number of neopixels on the device
            const N_LEDS: u16 = 1;

            maybe_async! {

                $async @fn enable_neopixel(&mut self) -> Result<(), SeesawError<D::I2cError>> {
                    let addr = self.addr();

                    maybe_async!(
                            self.driver()
                                .write_u8(addr, SET_PIN, Self::PIN)
                        => $async.await)?;
                    maybe_async!(
                            self.driver().delay().delay_us(10_000)
                        => $async.await);
                    maybe_async!(
                            self.driver().write_u16(addr, SET_LEN, 3 * Self::N_LEDS)
                        => $async.await)?;
                    maybe_async!(
                            self.driver().delay().delay_us(10_000)
                        => $async.await);
                    Ok(())
                }

                $async @fn set_neopixel_speed(&mut self, speed: NeopixelSpeed) -> Result<(), SeesawError<D::I2cError>> {
                    let addr = self.addr();

                    let speed_bit = match speed {
                        NeopixelSpeed::Khz400 => 0,
                        NeopixelSpeed::Khz800 => 1,
                    };
                    maybe_async!(
                            self.driver()
                                .write_u8(addr, SET_SPEED, speed_bit)
                        => $async.await)?;
                    maybe_async!(
                            self.driver().delay().delay_us(10_000)
                        => $async.await);
                    Ok(())
                }

                $async @fn set_neopixel_color(&mut self, r: u8, g: u8, b: u8) -> Result<(), SeesawError<D::I2cError>> {
                    maybe_async!(
                            self.set_nth_neopixel_color(0, r, g, b)
                        => $async.await)
                }

                $async @fn set_nth_neopixel_color(
                    &mut self,
                    n: u16,
                    r: u8,
                    g: u8,
                    b: u8,
                ) -> Result<(), SeesawError<D::I2cError>> {
                    assert!(n < Self::N_LEDS);
                    let [zero, one] = u16::to_be_bytes(3 * n);
                    let addr = self.addr();

                    maybe_async!(
                            self.driver()
                                .register_write(addr, SET_BUF, &[zero, one, r, g, b, 0x00])
                        => $async.await)?;
                    Ok(())
                }

                $async @fn set_neopixel_colors(
                    &mut self,
                    colors: &[(u8, u8, u8); Self::N_LEDS as usize],
                ) -> Result<(), SeesawError<D::I2cError>>
                where
                    [(); Self::N_LEDS as usize]: Sized,
                {
                    let addr = self.addr();

                    for n in 0..Self::N_LEDS {
                        let [zero, one] = u16::to_be_bytes(3 * n);
                        let color = colors[n as usize];
                        maybe_async!(
                                self.driver().register_write(
                                    addr,
                                    SET_BUF,
                                    &[zero, one, color.0, color.1, color.2, 0x00],
                                )
                            => $async.await)?;
                    }
                    Ok(())
                }

                $async @fn sync_neopixel(&mut self) -> Result<(), SeesawError<D::I2cError>> {
                    let addr = self.addr();

                    maybe_async!(
                            self.driver()
                                .register_write(addr, SHOW, &[])
                        => $async.await)?;
                    maybe_async!(
                            self.driver().delay().delay_us(125)
                        => $async.await);
                    Ok(())
                }
            }
        }
    };
}

neopixel_def! {
    for blocking;
    neopixel { NeopixelModule<Driver> }
    seesaw_device { SeesawDevice }
}
neopixel_def! {
    for async;
    neopixel { NeopixelModuleAsync<DriverAsync> }
    seesaw_device { SeesawDeviceAsync }
}

/// NeopixelModule: The Neopixel protocol speed
#[derive(Debug, Default)]
pub enum NeopixelSpeed {
    Khz400 = 0,
    #[default]
    Khz800 = 1,
}
