#[macro_export(local_inner_macros)]
macro_rules! seesaw_device {
    (
        $(#[$attr:meta])*
        name: $name:ident,
        hardware_id: $hardware_id:expr,
        product_id: $product_id:expr,
        default_addr: $default_addr:expr,
        modules: [
            $($module_name:ident $({
                $($const_name:ident: $const_value:expr $(,)?),*
            })?),*
            $(,)?
        ]
         $(,)?
    ) => {
        $(#[$attr])*
        ///
        #[doc=core::concat!("[Adafruit Product Page](https://www.adafruit.com/product/", core::stringify!($product_id),")")]
        #[derive(Debug)]
        pub struct $name<D>(u8, D);

        impl $name<()> {
            pub const fn default_addr() -> u8 {
                $default_addr
            }
            pub const fn hardware_id() -> $crate::HardwareId {
                $hardware_id
            }
            pub const fn product_id() -> u16 {
                $product_id
            }
            pub fn new(addr: u8) -> Self {
                Self(addr, ())
            }
            pub fn new_with_default_addr() -> Self {
                Self(Self::default_addr(), ())
            }
            pub fn with_driver<D: $crate::Driver>(&self, driver: D) -> $name<D> {
                $name(self.0, driver)
            }
            pub fn with_driver_async<D: $crate::DriverAsync>(&self, driver: D) -> $name<D> {
                $name(self.0, driver)
            }
        }

        // TODO: consider def macro for identical implementations
        impl<D: $crate::Driver> $crate::SeesawDevice for $name<D> {
            type Driver = D;
            type Error = $crate::SeesawError<D::I2cError>;
            const DEFAULT_ADDR: u8 = $default_addr;
            const HARDWARE_ID: $crate::HardwareId = $hardware_id;
            const PRODUCT_ID: u16 = $product_id;

            fn addr(&self) -> u8 {
                self.0
            }

            fn driver(&mut self) -> &mut D {
                &mut self.1
            }
        }
        impl<D: $crate::DriverAsync> $crate::SeesawDeviceAsync for $name<D> {
            type Driver = D;
            type Error = $crate::SeesawError<D::I2cError>;
            const DEFAULT_ADDR: u8 = $default_addr;
            const HARDWARE_ID: $crate::HardwareId = $hardware_id;
            const PRODUCT_ID: u16 = $product_id;

            fn addr(&self) -> u8 {
                self.0
            }

            fn driver(&mut self) -> &mut D {
                &mut self.1
            }
        }

        $(
            impl_device_module! { $name, $module_name $({$($const_name: $const_value),*})* }
        )*
    };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
// TODO: add async impls for all
macro_rules! impl_device_module {
    ($device:ident, AdcModule $({})?) => {
        impl<D: $crate::Driver> $crate::modules::adc::AdcModule<D> for $device<D> {}
    };
    ($device:ident, EncoderModule { button_pin: $button_pin:expr }) => {
        impl<D: $crate::Driver> $crate::modules::encoder::EncoderModule<D> for $device<D> {
            const ENCODER_BTN_PIN: u8 = $button_pin;
        }
        impl<D: $crate::DriverAsync> $crate::modules::encoder::EncoderModuleAsync<D>
            for $device<D>
        {
            const ENCODER_BTN_PIN: u8 = $button_pin;
        }
    };
    ($device:ident, GpioModule $({})?) => {
        impl<D: $crate::Driver> $crate::modules::gpio::GpioModule<D> for $device<D> {}
        impl<D: $crate::DriverAsync> $crate::modules::gpio::GpioModuleAsync<D> for $device<D> {}
    };
    ($device:ident, NeopixelModule { num_leds: $num_leds:expr, pin: $pin:expr }) => {
        impl<D: $crate::Driver> $crate::modules::neopixel::NeopixelModule<D> for $device<D> {
            const N_LEDS: u16 = $num_leds;
            const PIN: u8 = $pin;
        }
    };
    ($device:ident, StatusModule $({})?) => {
        impl<D: $crate::Driver> $crate::modules::StatusModule<D> for $device<D> {}
    };
    ($device:ident, TimerModule $({})?) => {
        impl<D: $crate::Driver> $crate::modules::timer::TimerModule<D> for $device<D> {}
    };
}

#[macro_export]
macro_rules! maybe_async {
    ($(async @fn $name:ident ( $($args:tt)* ) -> $return:ty);+ $(;)?) => {
        $(async fn $name ( $($args)* ) -> $return;)+
    };
    ($(blocking @fn $name:ident ( $($args:tt)* ) -> $return:ty);+ $(;)?) => {
        $(fn $name ( $($args)* ) -> $return;)+
    };
    ($(async @fn $name:ident ( $($args:tt)* ) -> $return:ty $block:block)+) => {
        $(async fn $name ( $($args)* ) -> $return $block)+
    };
    ($(blocking @fn $name:ident ( $($args:tt)* ) -> $return:ty $block:block)+) => {
        $(fn $name ( $($args)* ) -> $return $block)+
    };
    ($expr:expr => async.await ) => {
        $expr.await
    };
    ($expr:expr => blocking.await ) => {
        $expr
    };
}
