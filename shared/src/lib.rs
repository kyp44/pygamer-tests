#![no_std]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use display::DisplayTextStyle;
use embedded_graphics::{
    mono_font::{self, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text,
};
use pygamer::{
    hal::{clock::GenericClockController, delay::Delay, time::Hertz},
    pac, ButtonReader, DisplayDriver, Keys, Pins, RedLed,
};

mod display;
#[cfg(feature = "rtic")]
mod monotonic;

#[cfg(feature = "neopixels")]
pub type NeoPixelsDriver = ws2812_spi::Ws2812<pygamer::pins::NeopixelSpi>;

pub static DISPLAY_SIZE: Size = Size::new(160, 128);
pub const FONT: mono_font::MonoFont = mono_font::ascii::FONT_5X8;
pub const BACKGROUND_COLOR: Rgb565 = Rgb565::WHITE;

#[cfg(feature = "clock1k")]
pub const RTC_CLOCK_RATE: Hertz = Hertz::from_raw(1024);
#[cfg(feature = "clock32k")]
pub const RTC_CLOCK_RATE: Hertz = Hertz::from_raw(32768);

pub static TEXT_STYLE: MonoTextStyle<Rgb565> = mono_font::MonoTextStyleBuilder::new()
    .font(&crate::FONT)
    .text_color(Rgb565::BLACK)
    .background_color(BACKGROUND_COLOR)
    .build();

lazy_static::lazy_static! {
    pub static ref DISPLAY_TEXT_STYLE: DisplayTextStyle<Rgb565> = DisplayTextStyle::new(
        Point::zero(),
        Some(DISPLAY_SIZE),
        TEXT_STYLE,
        text::TextStyleBuilder::new()
            .baseline(text::Baseline::Top)
            .build(),
    );
}

pub mod prelude {
    pub use super::display::*;
    #[cfg(feature = "rtic")]
    pub use super::monotonic::{display_monotonic_info, Mono};
    pub use super::ButtonReaderExt;
    #[cfg(feature = "neopixels")]
    pub use super::NeoPixelsDriver;
    pub use super::{
        setup, SetupPackage, BACKGROUND_COLOR, DISPLAY_SIZE, DISPLAY_TEXT_STYLE, FONT,
        RTC_CLOCK_RATE, TEXT_STYLE,
    };
    pub use core::fmt::Write;
    pub use hal::prelude::*;
    pub use lazy_static::lazy_static;
    #[cfg(feature = "neopixels")]
    pub use smart_leds::{SmartLedsWrite, RGB8};

    // Re-exports
    pub use cortex_m;
    pub use embedded_graphics;
    pub use pygamer::{self, hal, pac};
    #[cfg(feature = "rtic")]
    pub use rtic;
}

pub trait ButtonReaderExt {
    /// Blocks until the A or Start button are pressed.
    fn wait_for_button(&mut self);
}
impl ButtonReaderExt for ButtonReader {
    fn wait_for_button(&mut self) {
        'main: loop {
            for event in self.events() {
                match event {
                    Keys::StartDown | Keys::ADown => break 'main,
                    _ => {}
                }
            }
        }
    }
}

pub struct SetupPackage {
    pub delay: Delay,
    pub display: DisplayDriver,
    pub button_reader: ButtonReader,
    #[cfg(feature = "neopixels")]
    pub neopixels: NeoPixelsDriver,
    pub red_led: RedLed,
    pub tc4: pac::Tc4,
    pub rtc: pac::Rtc,
    pub clocks: GenericClockController,
    pub mclk: pac::Mclk,
    pub osc32kctrl: pac::Osc32kctrl,
}

#[inline]
pub fn setup(mut peripherals: pac::Peripherals, core: pac::CorePeripherals) -> SetupPackage {
    // NOTE: We would like to use the v2 of the clock module, but this is not yet integrated
    // into the rest of the HAL or the `pygamer` BSP. For example, the display `init` method
    // below requires clock v1 parameters.
    /* let (mut buses, clocks, tokens) = clock_system_at_reset(
        peripherals.oscctrl,
        peripherals.osc32kctrl,
        peripherals.gclk,
        peripherals.mclk,
        &mut peripherals.nvmctrl,
    ); */

    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.gclk,
        &mut peripherals.mclk,
        &mut peripherals.osc32kctrl,
        &mut peripherals.oscctrl,
        &mut peripherals.nvmctrl,
    );

    let pins = Pins::new(peripherals.port).split();
    let mut delay = Delay::new(core.SYST, &mut clocks);
    // Here is how this can be initialized using the clock v2 API instead
    //let mut delay = Delay::new_with_source(core.SYST, clocks.gclk0);

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

    display.clear(BACKGROUND_COLOR).unwrap();

    // NOTE: Selecting the RTC clock requires the clocks v2 API on SAMx5x chips!
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
        button_reader: pins.buttons.init(),
        #[cfg(feature = "neopixels")]
        neopixels,
        red_led: pins.led_pin.into(),
        tc4: peripherals.tc4,
        rtc: peripherals.rtc,
        clocks,
        mclk: peripherals.mclk,
        osc32kctrl: peripherals.osc32kctrl,
    }
}
