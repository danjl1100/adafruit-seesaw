use crate::{
    common::{Modules, Reg},
    maybe_async, Driver, DriverAsync, DriverExt, DriverExtAsync, HardwareId, SeesawDevice,
    SeesawDeviceAsync,
};

/// WO - 16 bits
/// The first byte of the register indicates which PWM pin will have its value
/// set The second byte is the actual PWM value
const PWM_VAL: &Reg = &[Modules::Timer.into_u8(), 0x01];

macro_rules! timer_module_def {
    (
        for $async:ident;
        timer_module { $name:ident < $driver:ident > }
        seesaw_device { $seesaw_device:ident }
    ) => {
        /// The PWM module provides up to 4 8-bit PWM outputs.
        /// The module base register address for the PWM module is 0x08.
        /// PWM outputs are available on pins PA04, PA05, PA06, and PA07.
        pub trait $name<D: $driver>: $seesaw_device<Driver = D> {

            maybe_async! {

                $async @fn analog_write(&mut self, pin: u8, value: u8) -> Result<(), crate::SeesawError<D::I2cError>> {
                    let mapped_pin = match Self::HARDWARE_ID {
                        HardwareId::ATTINY817 => pin,
                        HardwareId::SAMD09 => match pin {
                            4 => 0,
                            5 => 1,
                            6 => 2,
                            7 => 3,
                            _ => 0,
                        },
                    };

                    let addr = self.addr();
                    maybe_async!(
                            self.driver()
                                .write_u16(addr, PWM_VAL, u16::from_be_bytes([mapped_pin, value]))
                        => $async.await)?;
                    Ok(())
                }

            }
        }

    };
}
timer_module_def! {
    for blocking;
    timer_module { TimerModule<Driver> }
    seesaw_device { SeesawDevice }
}
timer_module_def! {
    for async;
    timer_module { TimerModuleAsync<DriverAsync> }
    seesaw_device { SeesawDeviceAsync }
}
