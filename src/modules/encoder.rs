use super::gpio::{GpioModule, GpioModuleAsync, PinMode};
use crate::{
    common::{Modules, Reg},
    maybe_async, DelayAsync, DriverExt, DriverExtAsync,
};
use embedded_hal::delay::DelayUs;

#[allow(dead_code)]
const STATUS: &Reg = &[Modules::Encoder.into_u8(), 0x00];
const INT_SET: &Reg = &[Modules::Encoder.into_u8(), 0x10];
const INT_CLR: &Reg = &[Modules::Encoder.into_u8(), 0x20];
const POSITION: &Reg = &[Modules::Encoder.into_u8(), 0x30];
const DELTA: &Reg = &[Modules::Encoder.into_u8(), 0x40];

macro_rules! encode_module_def {
    (
        for $async:ident;
        module { $module_name:ident < $driver:path > }
        gpio { $gpio:ident }
    ) => {
        pub trait $module_name<D: $driver>: $gpio<D> {
            const ENCODER_BTN_PIN: u8;

            maybe_async! {

                $async @fn enable_button(&mut self) -> Result<(), crate::SeesawError<D::I2cError>> {
                    maybe_async!(
                            self.set_pin_mode(Self::ENCODER_BTN_PIN, PinMode::InputPullup)
                         => $async.await)?;
                    self.driver().delay().delay_us(125);
                    Ok(())
                }

                $async @fn button(&mut self) -> Result<bool, crate::SeesawError<D::I2cError>> {
                    maybe_async!(
                            self.digital_read(Self::ENCODER_BTN_PIN)
                        => $async.await)
                }

                $async @fn delta(&mut self) -> Result<i32, crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();
                    let delta = maybe_async!(
                            self.driver().read_i32(addr, DELTA)
                        => $async.await)?;
                    Ok(delta)
                }

                $async @fn disable_interrupt(&mut self) -> Result<(), crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();
                    maybe_async!(
                            self.driver().write_u8(addr, INT_CLR, 1)
                        => $async.await)?;
                    Ok(())
                }

                $async @fn enable_interrupt(&mut self) -> Result<(), crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();
                    maybe_async!(
                            self.driver().write_u8(addr, INT_SET, 1)
                        => $async.await)?;
                    Ok(())
                }

                $async @fn position(&mut self) -> Result<i32, crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();
                    let position = maybe_async!(
                            self.driver().read_i32(addr, POSITION)
                        => $async.await)?;
                    Ok(position)
                }

                $async @fn set_position(&mut self, pos: i32) -> Result<(), crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();
                    maybe_async!(
                            self.driver().write_i32(addr, POSITION, pos)
                        => $async.await)?;
                    Ok(())
                }
            }
        }
    };
}
encode_module_def! {
    for blocking;
    module { EncoderModule <crate::Driver> }
    gpio { GpioModule }
}
encode_module_def! {
    for async;
    module { EncoderModuleAsync <crate::DriverAsync> }
    gpio { GpioModuleAsync }
}
