macro_rules! driver_def {
    (
        for $async:ident;
        scope { $scope:ident }
        i2c { $i2c_driver:ident : $i2c_trait:path, Err($i2c_error:path) }
        driver { $driver:ident }
        driver_ext { $driver_ext:ident }
        delay_trait { $delay_trait:path }
    ) => {
        pub use $scope::{$driver, $driver_ext, $i2c_driver};
        mod $scope {
            use super::SevenBitAddress;
            use crate::{common::Reg, maybe_async};
            use $delay_trait as _;
            use $i2c_trait as _;

            const DELAY_TIME_MICROS: u32 = 125;

            /// Blanket trait for something that implements I2C bus operations, with a
            /// combined Error associated type
            #[doc(hidden)]
            pub trait $i2c_driver: $i2c_trait {
                type I2cError: From<Self::Error>;
            }

            impl<T> $i2c_driver for T
            where
                T: $i2c_trait,
            {
                type I2cError = T::Error;
            }

            pub trait $driver {
                type I2cError: From<<Self::I2c as $i2c_error>::Error>;
                type I2c: $i2c_driver;
                type Delay: $delay_trait;
                fn i2c(&mut self) -> &mut Self::I2c;
                fn delay(&mut self) -> &mut Self::Delay;
            }
            impl<T> $driver for T
            where
                T: $i2c_driver + $delay_trait,
            {
                type Delay = Self;
                type I2c = Self;
                type I2cError = <T as $i2c_error>::Error;

                fn i2c(&mut self) -> &mut Self::I2c {
                    self
                }

                fn delay(&mut self) -> &mut Self::Delay {
                    self
                }
            }

            macro_rules! impl_integer_write {
                ($fn:ident $nty:tt) => {
                    maybe_async! {
                        $async @fn $fn(
                            &mut self,
                            addr: SevenBitAddress,
                            reg: &Reg,
                            value: $nty,
                        ) -> Result<(), Self::Error> {
                            maybe_async!(
                                    self.register_write(addr, reg, &<$nty>::to_be_bytes(value))
                                => $async.await)
                        }
                    }
                };
            }

            macro_rules! impl_integer_read {
                ($fn:ident $nty:tt) => {
                    maybe_async! {
                        $async @fn $fn(
                            &mut self,
                            addr: SevenBitAddress,
                            reg: &Reg,
                        ) -> Result<$nty, Self::Error> {
                            maybe_async!(
                                    self.register_read::<{ ($nty::BITS / 8) as usize }>(addr, reg)
                                => $async.await)
                                .map($nty::from_be_bytes)
                        }
                    }
                };
            }

            pub trait $driver_ext {
                type Error;

                maybe_async! {
                    $async @fn register_read<const N: usize>(
                        &mut self,
                        addr: SevenBitAddress,
                        reg: &Reg,
                    ) -> Result<[u8; N], Self::Error>;

                    $async @fn register_write<const N: usize>(
                        &mut self,
                        addr: u8,
                        reg: &Reg,
                        bytes: &[u8; N],
                    ) -> Result<(), Self::Error>
                    where
                        [(); N + 2]: Sized;
                }

                impl_integer_read! { read_u8 u8 }
                impl_integer_read! { read_u16 u16 }
                impl_integer_read! { read_u32 u32 }
                impl_integer_read! { read_u64 u64 }
                impl_integer_read! { read_i8 i8 }
                impl_integer_read! { read_i16 i16 }
                impl_integer_read! { read_i32 i32 }
                impl_integer_read! { read_i64 i64 }
                impl_integer_write! { write_u8 u8 }
                impl_integer_write! { write_u16 u16 }
                impl_integer_write! { write_u32 u32 }
                impl_integer_write! { write_u64 u64 }
                impl_integer_write! { write_i8 i8 }
                impl_integer_write! { write_i16 i16 }
                impl_integer_write! { write_i32 i32 }
                impl_integer_write! { write_i64 i64 }
            }

            impl<T: $driver> $driver_ext for T {
                type Error = T::I2cError;

                maybe_async! {
                    $async @fn register_read<const N: usize>(
                        &mut self,
                        addr: SevenBitAddress,
                        reg: &Reg,
                    ) -> Result<[u8; N], Self::Error> {
                        let mut buffer = [0u8; N];
                        maybe_async!(
                                self.i2c().write(addr, reg)
                            => $async.await)?;
                        maybe_async!(
                                self.delay().delay_us(DELAY_TIME_MICROS)
                            => $async.await);
                        maybe_async!(
                                self.i2c().read(addr, &mut buffer)
                            => $async.await)?;
                        Ok(buffer)
                    }

                    $async @fn register_write<const N: usize>(
                        &mut self,
                        addr: SevenBitAddress,
                        reg: &Reg,
                        bytes: &[u8; N],
                    ) -> Result<(), Self::Error>
                    where
                        [(); N + 2]: Sized,
                    {
                        let mut buffer = [0u8; N + 2];
                        buffer[0..2].copy_from_slice(reg);
                        buffer[2..].copy_from_slice(bytes);

                        maybe_async!(
                                self.i2c().write(addr, &buffer)
                            => $async.await)?;
                        maybe_async!(
                                self.delay().delay_us(DELAY_TIME_MICROS)
                            => $async.await);
                        Ok(())
                    }
                }
            }
        }
    };
}

type SevenBitAddress = embedded_hal::i2c::SevenBitAddress;

driver_def! {
    for blocking;
    scope { blocking }
    i2c { I2cDriver: embedded_hal::i2c::I2c, Err(embedded_hal::i2c::ErrorType) }
    driver { Driver }
    driver_ext { DriverExt }
    delay_trait { embedded_hal::delay::DelayUs }
}
driver_def! {
    for async;
    scope { non_blocking }
    i2c { I2cDriverAsync: embedded_hal_async::i2c::I2c, Err(embedded_hal_async::i2c::ErrorType) }
    driver { DriverAsync }
    driver_ext { DriverExtAsync }
    delay_trait { crate::driver::DelayAsync }
}

pub trait DelayAsync {
    async fn delay_us(&mut self, duration_micros: u32);
}
