const DELAY_TIME_MICROS: u32 = 125;

pub use blocking::{Driver, DriverExt, I2cDriver};
mod blocking {
    use super::DELAY_TIME_MICROS;
    use crate::common::Reg;
    use embedded_hal::{
        delay::{self, DelayUs},
        i2c::{self, I2c},
    };

    /// Blanket trait for something that implements I2C bus operations, with a
    /// combined Error associated type
    #[doc(hidden)]
    pub trait I2cDriver: i2c::I2c {
        type I2cError: From<Self::Error>;
    }

    impl<T> I2cDriver for T
    where
        T: i2c::I2c,
    {
        type I2cError = T::Error;
    }

    pub trait Driver {
        type I2cError: From<<Self::I2c as i2c::ErrorType>::Error>;
        type I2c: I2cDriver;
        type Delay: delay::DelayUs;
        fn i2c(&mut self) -> &mut Self::I2c;
        fn delay(&mut self) -> &mut Self::Delay;
    }
    impl<T> Driver for T
    where
        T: I2cDriver + delay::DelayUs,
    {
        type Delay = Self;
        type I2c = Self;
        type I2cError = <T as i2c::ErrorType>::Error;

        fn i2c(&mut self) -> &mut Self::I2c {
            self
        }

        fn delay(&mut self) -> &mut Self::Delay {
            self
        }
    }

    macro_rules! impl_integer_write {
        ($fn:ident $nty:tt) => {
            fn $fn(
                &mut self,
                addr: i2c::SevenBitAddress,
                reg: &Reg,
                value: $nty,
            ) -> Result<(), Self::Error> {
                self.register_write(addr, reg, &<$nty>::to_be_bytes(value))
            }
        };
    }

    macro_rules! impl_integer_read {
        ($fn:ident $nty:tt) => {
            fn $fn(&mut self, addr: i2c::SevenBitAddress, reg: &Reg) -> Result<$nty, Self::Error> {
                self.register_read::<{ ($nty::BITS / 8) as usize }>(addr, reg)
                    .map($nty::from_be_bytes)
            }
        };
    }

    pub trait DriverExt {
        type Error;

        fn register_read<const N: usize>(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
        ) -> Result<[u8; N], Self::Error>;

        fn register_write<const N: usize>(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
            bytes: &[u8; N],
        ) -> Result<(), Self::Error>
        where
            [(); N + 2]: Sized;

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

    impl<T: Driver> DriverExt for T {
        type Error = T::I2cError;

        fn register_read<const N: usize>(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
        ) -> Result<[u8; N], Self::Error> {
            let mut buffer = [0u8; N];
            self.i2c().write(addr, reg)?;
            self.delay().delay_us(DELAY_TIME_MICROS);
            self.i2c().read(addr, &mut buffer)?;
            Ok(buffer)
        }

        fn register_write<const N: usize>(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
            bytes: &[u8; N],
        ) -> Result<(), Self::Error>
        where
            [(); N + 2]: Sized,
        {
            let mut buffer = [0u8; N + 2];
            buffer[0..2].copy_from_slice(reg);
            buffer[2..].copy_from_slice(bytes);

            self.i2c().write(addr, &buffer)?;
            self.delay().delay_us(DELAY_TIME_MICROS);
            Ok(())
        }
    }
}
pub use non_blocking::{DelayAsync, DriverAsync, DriverExtAsync, I2cDriverAsync};
mod non_blocking {
    use super::DELAY_TIME_MICROS;
    use crate::common::Reg;
    use embedded_hal_async::i2c::{self, I2c};

    /// Blanket trait for something that implements I2C bus operations, with a
    /// combined Error associated type
    #[doc(hidden)]
    pub trait I2cDriverAsync: i2c::I2c {
        type I2cError: From<Self::Error>;
    }

    impl<T> I2cDriverAsync for T
    where
        T: i2c::I2c,
    {
        type I2cError = T::Error;
    }

    pub trait DelayAsync {
        async fn delay_us(&mut self, duration_micros: u32);
    }

    pub trait DriverAsync {
        type Delay: DelayAsync;
        type I2cError: From<<Self::I2c as i2c::ErrorType>::Error>;
        type I2c: I2cDriverAsync;
        fn i2c(&mut self) -> &mut Self::I2c;
        fn delay(&mut self) -> &mut Self::Delay;
    }

    macro_rules! impl_integer_write {
        ($fn:ident $nty:tt) => {
            async fn $fn(
                &mut self,
                addr: i2c::SevenBitAddress,
                reg: &Reg,
                value: $nty,
            ) -> Result<(), Self::Error> {
                self.register_write(addr, reg, &<$nty>::to_be_bytes(value))
                    .await
            }
        };
    }

    macro_rules! impl_integer_read {
        ($fn:ident $nty:tt) => {
            async fn $fn(
                &mut self,
                addr: i2c::SevenBitAddress,
                reg: &Reg,
            ) -> Result<$nty, Self::Error> {
                self.register_read::<{ ($nty::BITS / 8) as usize }>(addr, reg)
                    .await
                    .map($nty::from_be_bytes)
            }
        };
    }

    pub trait DriverExtAsync {
        type Error;

        async fn register_read<const N: usize>(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
        ) -> Result<[u8; N], Self::Error>;

        async fn register_write<const N: usize>(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
            bytes: &[u8; N],
        ) -> Result<(), Self::Error>
        where
            [(); N + 2]: Sized;

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

    // TODO: consider a def macro for DriverExt implementations, to ensure identical
    impl<T: DriverAsync> DriverExtAsync for T {
        type Error = T::I2cError;

        async fn register_read<const N: usize>(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
        ) -> Result<[u8; N], Self::Error> {
            let mut buffer = [0u8; N];
            self.i2c().write(addr, reg).await?;
            self.delay().delay_us(DELAY_TIME_MICROS).await;
            self.i2c().read(addr, &mut buffer).await?;
            Ok(buffer)
        }

        async fn register_write<const N: usize>(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
            bytes: &[u8; N],
        ) -> Result<(), Self::Error>
        where
            [(); N + 2]: Sized,
        {
            let mut buffer = [0u8; N + 2];
            buffer[0..2].copy_from_slice(reg);
            buffer[2..].copy_from_slice(bytes);

            self.i2c().write(addr, &buffer).await?;
            self.delay().delay_us(DELAY_TIME_MICROS).await;
            Ok(())
        }
    }
}
