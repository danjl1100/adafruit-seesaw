use crate::{
    common::{Modules, Reg},
    maybe_async, Driver, DriverAsync, DriverExt, DriverExtAsync, HardwareId, SeesawDevice,
    SeesawDeviceAsync,
};

/// RO - 8 bits
#[allow(dead_code)]
const STATUS: &Reg = &[Modules::Adc.into_u8(), 0x00];

/// WO - 8 bits
/// Writing a 1 to any bit in this register enables the corresponding interrupt.
/// Writing zeros to this register has no effect.
#[allow(dead_code)]
const INTENSET: &Reg = &[Modules::Adc.into_u8(), 0x02];

/// NOT SUPPORTED BY SEESAW PLATFORM
///
/// WO - 8 bits
/// Writing a 1 to any bit in this register enables the corresponding interrupt.
/// Writing zeros to this register has no effect.
#[allow(dead_code)]
const INTENCLR: &Reg = &[Modules::Adc.into_u8(), 0x03];

/// NOT SUPPORTED BY SEESAW PLATFORM
///
/// WO
/// Writing 1 to this register sets window control.
#[allow(dead_code)]
const WINMODE: &Reg = &[Modules::Adc.into_u8(), 0x04];

/// NOT SUPPORTED BY SEESAW PLATFORM
///
/// WO - 32 bits
/// This register sets the threshold values for window mode.
/// B31 - B16: High threshold
/// B15 - B0: Low threshold
#[allow(dead_code)]
const WINTHRESH: &Reg = &[Modules::Adc.into_u8(), 0x05];

/// RO - 16bits
/// ADC value for channel 0
const CHANNEL_0: &Reg = &[Modules::Adc.into_u8(), 0x07];

macro_rules! adc_module_def {
    (
        for $async:ident;
        adc_module { $name:ident < $driver:path > }
        seesaw_device { $seesaw_device:ident }
    ) => {
        /// The ADC provides the ability to measure analog voltages at 10-bit
        /// resolution. The SAMD09 seesaw has 4 ADC inputs, the Attiny8x7 has 11 ADC
        /// inputs.
        ///
        /// The module base register address for the ADC is 0x09
        ///
        /// Conversions can be read by reading the corresponding CHANNEL register.
        ///
        /// When reading ADC data, there should be at least a 500 uS delay between
        /// writing the register number you would like to read from and attempting to
        /// read the data.
        ///
        /// Allow a delay of at least 1ms in between sequential ADC reads on different
        /// channels.
        pub trait $name<D: $driver>: $seesaw_device<Driver = D> {
            maybe_async! {
                $async @fn analog_read(
                    &mut self,
                    pin: u8,
                ) -> Result<u16, crate::SeesawError<D::I2cError>> {
                    let pin_offset = pin_offset(Self::HARDWARE_ID, pin);

                    let addr = self.addr();
                    let analog = maybe_async!(
                            self.driver()
                                .read_u16(addr, &[CHANNEL_0[0], CHANNEL_0[1] + pin_offset])
                        => $async.await)?;
                    Ok(analog)
                }
            }
        }
    };
}
adc_module_def! {
    for blocking;
    adc_module { AdcModule<Driver> }
    seesaw_device { SeesawDevice }
}
adc_module_def! {
    for async;
    adc_module { AdcModuleAsync<DriverAsync> }
    seesaw_device { SeesawDeviceAsync }
}

fn pin_offset(hardware_id: HardwareId, pin: u8) -> u8 {
    match hardware_id {
        HardwareId::ATTINY817 => pin,
        HardwareId::SAMD09 => match pin {
            2 => 0,
            3 => 1,
            4 => 2,
            5 => 3,
            _ => 0,
        },
    }
}
