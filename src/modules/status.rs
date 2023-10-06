use crate::{
    driver::Driver, maybe_async, DelayAsync, DriverAsync, DriverExt, DriverExtAsync, Modules, Reg,
    SeesawDevice, SeesawDeviceAsync,
};
use embedded_hal::delay::DelayUs;

const STATUS_HW_ID: &Reg = &[Modules::Status.into_u8(), 0x01];
const STATUS_VERSION: &Reg = &[Modules::Status.into_u8(), 0x02];
const STATUS_OPTIONS: &Reg = &[Modules::Status.into_u8(), 0x03];
const STATUS_TEMP: &Reg = &[Modules::Status.into_u8(), 0x04];
const STATUS_SWRST: &Reg = &[Modules::Status.into_u8(), 0x7F];

macro_rules! seesaw_device_def {
    (
       for $async:ident;
       status_module { $status_module_name:ident < $driver:path > }
       seesaw_device { $seesaw_device_name:ident }
   ) => {
        pub trait $status_module_name<D: $driver>: $seesaw_device_name<Driver = D> {
            maybe_async! {
                $async @fn capabilities(
                    &mut self,
                ) -> Result<DeviceCapabilities, crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();

                    let cap = maybe_async!(
                            self.driver().read_u32(addr, STATUS_OPTIONS)
                        => $async.await)?;
                    Ok(DeviceCapabilities::from(cap))
                }

                $async @fn hardware_id(&mut self) -> Result<u8, crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();
                    let hardware_id = maybe_async!(
                            self.driver().read_u8(addr, STATUS_HW_ID)
                        => $async.await)?;
                    Ok(hardware_id)
                }

                $async @fn product_info(
                    &mut self,
                ) -> Result<ProductDateCode, crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();

                    let version = maybe_async!(
                            self.driver().read_u32(addr, STATUS_VERSION)
                        => $async.await)?;
                    Ok(ProductDateCode::from(version))
                }

                $async @fn reset(&mut self) -> Result<(), crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();

                    maybe_async!(
                            self.driver().write_u8(addr, STATUS_SWRST, 0xFF)
                        => $async.await)?;
                    maybe_async!(
                            self.driver().delay().delay_us(125_000)
                        => $async.await);
                    Ok(())
                }

                $async @fn reset_and_verify_seesaw(
                    &mut self,
                ) -> Result<(), crate::SeesawError<D::I2cError>> {
                    let hw_id = Self::HARDWARE_ID;
                    maybe_async!(self.reset() => $async.await)?;
                    let current_id = maybe_async!(self.hardware_id() => $async.await)?;
                    if current_id == hw_id.into() {
                        Ok(())
                    } else {
                        Err(crate::SeesawError::InvalidHardwareId(current_id))
                    }
                }

                $async @fn temp(&mut self) -> Result<f32, crate::SeesawError<D::I2cError>> {
                    let addr = self.addr();

                    let temp_buf = maybe_async!(self.driver().read_u32(addr, STATUS_TEMP) => $async.await)?;
                    Ok(temp_buf as f32 / (1u32 << 16) as f32)
                }
            }
        }
    };
}

seesaw_device_def! {
    for blocking;
    status_module { StatusModule<Driver> }
    seesaw_device { SeesawDevice }
}
seesaw_device_def! {
    for async;
    status_module { StatusModuleAsync<DriverAsync> }
    seesaw_device { SeesawDeviceAsync }
}

/// StatusModule
#[derive(Copy, Clone, Debug)]
pub struct DeviceCapabilities {
    pub adc: bool,
    pub dac: bool,
    pub dap: bool,
    pub eeprom: bool,
    pub encoder: bool,
    pub gpio: bool,
    pub interrupt: bool,
    pub keypad: bool,
    pub neopixel: bool,
    pub sercom0: bool,
    pub spectrum: bool,
    pub status: bool,
    pub timer: bool,
    pub touch: bool,
}

impl From<u32> for DeviceCapabilities {
    fn from(value: u32) -> Self {
        DeviceCapabilities {
            adc: value >> Modules::Adc as u8 & 1 == 1,
            dac: value >> Modules::Dac as u8 & 1 == 1,
            dap: value >> Modules::Dap as u8 & 1 == 1,
            eeprom: value >> Modules::Eeprom as u8 & 1 == 1,
            encoder: value >> Modules::Encoder as u8 & 1 == 1,
            gpio: value >> Modules::Gpio as u8 & 1 == 1,
            interrupt: value >> Modules::Interrupt as u8 & 1 == 1,
            keypad: value >> Modules::Keypad as u8 & 1 == 1,
            neopixel: value >> Modules::Neopixel as u8 & 1 == 1,
            sercom0: value >> Modules::Sercom0 as u8 & 1 == 1,
            spectrum: value >> Modules::Spectrum as u8 & 1 == 1,
            status: value >> Modules::Status as u8 & 1 == 1,
            timer: value >> Modules::Timer as u8 & 1 == 1,
            touch: value >> Modules::Touch as u8 & 1 == 1,
        }
    }
}

/// StatusModule
#[derive(Debug)]
pub struct ProductDateCode {
    pub id: u16,
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl From<u32> for ProductDateCode {
    fn from(vers: u32) -> Self {
        Self {
            id: (vers >> 16) as u16,
            year: ((vers & 0x3F) + 2000) as u16,
            month: ((vers >> 7) & 0xF) as u8,
            day: ((vers >> 11) & 0x1F) as u8,
        }
    }
}
