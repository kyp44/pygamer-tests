#![no_std]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use embedded_graphics::{
    mono_font::{self, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
};
use pygamer::{
    hal::{clock::GenericClockController, delay::Delay},
    pac, DisplayDriver, Pins, RedLed,
};

mod display;
#[cfg(feature = "rtic")]
mod monotonic;

#[cfg(feature = "neopixels")]
pub type NeoPixelsDriver = ws2812_spi::Ws2812<pygamer::pins::NeopixelSpi>;

pub static DISPLAY_SIZE: Size = Size::new(160, 128);
pub const FONT: mono_font::MonoFont = mono_font::ascii::FONT_5X8;

pub static DISPLAY_TEXT_STYLE: MonoTextStyle<Rgb565> = mono_font::MonoTextStyleBuilder::new()
    .font(&crate::FONT)
    .text_color(Rgb565::WHITE)
    .background_color(Rgb565::BLACK)
    .build();

pub mod prelude {
    pub use super::display::*;
    #[cfg(feature = "rtic")]
    pub use super::monotonic::{display_monotonic_info, Mono};
    #[cfg(feature = "neopixels")]
    pub use super::NeoPixelsDriver;
    pub use super::{setup, SetupPackage, DISPLAY_SIZE, DISPLAY_TEXT_STYLE, FONT};
    pub use core::fmt::Write;
    pub use fugit::{ExtU32, ExtU32Ceil, ExtU64, ExtU64Ceil, RateExtU32, RateExtU64};
    #[cfg(feature = "neopixels")]
    pub use smart_leds::{SmartLedsWrite, RGB8};

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
    #[cfg(feature = "neopixels")]
    pub neopixels: NeoPixelsDriver,
    pub red_led: RedLed,
    pub rtc: pac::Rtc,
    pub clocks: GenericClockController,
    pub mclk: pac::Mclk,
    pub osc32kctrl: pac::Osc32kctrl,
}

#[inline]
pub fn setup(mut peripherals: pac::Peripherals, core: pac::CorePeripherals) -> SetupPackage {
    // NOTE: Would like to use the v2 of the clock module, but this is not yet integrated
    // into the rest of the HAL.
    /* let (mut buses, clocks, tokens) = clock_system_at_reset(
        peripherals.oscctrl,
        peripherals.osc32kctrl,
        peripherals.gclk,
        peripherals.mclk,
        &mut peripherals.nvmctrl,
    ); */
    // TODO: Maybe we can use this with this delay instead since it should be compatible:
    // https://atsamd-rs.github.io/docs/samd51n/thumbv7em-none-eabihf/doc/atsamd_hal/delay/struct.Delay.html
    // There are also v1 compatibility conversions for these: https://atsamd-rs.github.io/docs/samd51n/thumbv7em-none-eabihf/doc/atsamd_hal/clock/v2/pclk/struct.Pclk.html

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

    // TOOD: Use HAL to do this instead of taking the PAC references, maybe clocks v2 to require proof.
    // Set the RTC clock source, which is no longer done within the monotonic.
    // TODO: Note that the clocks v1 API cannot select the internal 32k clock!
    #[cfg(feature = "clock1k")]
    peripherals
        .osc32kctrl
        .rtcctrl()
        .write(|w| w.rtcsel().ulp1k());
    #[cfg(feature = "clock32k")]
    peripherals
        .osc32kctrl
        .rtcctrl()
        .write(|w| w.rtcsel().ulp32k());

    #[cfg(feature = "neopixels")]
    let neopixels = pins.neopixel.init_spi(
        &mut clocks,
        // Unfortunately, the SPI driver requires a clock pin, even though it's not used by the
        // neopixels.
        pins.i2c.scl,
        peripherals.sercom2,
        &mut peripherals.mclk,
    );

    SetupPackage {
        delay,
        display,
        #[cfg(feature = "neopixels")]
        neopixels,
        red_led: pins.led_pin.into(),
        rtc: peripherals.rtc,
        clocks,
        mclk: peripherals.mclk,
        osc32kctrl: peripherals.osc32kctrl,
    }
}
