#![no_std]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use embedded_graphics::{mono_font, pixelcolor::Rgb565, prelude::*};
use pygamer::{
    hal::{clock::GenericClockController, delay::Delay},
    pac, DisplayDriver, Pins, RedLed,
};

mod display;
#[cfg(feature = "rtic")]
mod monotonic;

pub static DISPLAY_SIZE: Size = Size::new(160, 128);
pub const FONT: mono_font::MonoFont = mono_font::ascii::FONT_5X8;

pub mod prelude {
    pub use super::display::*;
    #[cfg(feature = "rtic")]
    pub use super::monotonic::Mono;
    pub use super::{setup, SetupPackage, DISPLAY_SIZE, FONT};
    pub use core::fmt::Write;
    pub use fugit::{ExtU32, ExtU32Ceil, ExtU64, ExtU64Ceil, RateExtU32, RateExtU64};

    // Re-exports
    pub use cortex_m;
    pub use embedded_graphics;
    pub use pygamer::{self, hal, pac};
    #[cfg(feature = "rtic")]
    pub use rtic;
}

pub struct SetupPackage {
    pub delay: Delay,
    pub display: DisplayDriver,
    pub red_led: RedLed,
    pub rtc: pac::Rtc,
    pub mclk: pac::Mclk,
    pub osc32kctrl: pac::Osc32kctrl,
}

#[inline]
pub fn setup(mut peripherals: pac::Peripherals, core: pac::CorePeripherals) -> SetupPackage {
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.gclk,
        &mut peripherals.mclk,
        &mut peripherals.osc32kctrl,
        &mut peripherals.oscctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = Pins::new(peripherals.port).split();
    let mut delay = Delay::new(core.SYST, &mut clocks);

    // Initialize the display
    let (mut display, _backlight) = pins
        .display
        .init(
            &mut clocks,
            peripherals.sercom4,
            &mut peripherals.mclk,
            peripherals.tc2,
            &mut delay,
        )
        .unwrap();

    display.clear(Rgb565::WHITE).unwrap();

    SetupPackage {
        delay,
        display,
        red_led: pins.led_pin.into(),
        rtc: peripherals.rtc,
        mclk: peripherals.mclk,
        osc32kctrl: peripherals.osc32kctrl,
    }
}
