#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use core::fmt::Write;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{mono_font, text};
use pac::{CorePeripherals, Peripherals};
use pygamer::hal::clock::GenericClockController;
use pygamer::hal::delay::Delay;
use pygamer::hal::ehal::delay::DelayNs;
use pygamer::{entry, pac, Pins};

mod panic_handler;

static DISPLAY_SIZE: Size = Size::new(160, 128);
const FONT: mono_font::MonoFont = mono_font::ascii::FONT_5X8;

pub use panic_handler::TextWriter;

#[entry]
fn main() -> ! {
    // Get the peripherals and pins and setup clocks
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
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

    write!(
        TextWriter::new(
            &mut display,
            Point::new(10, 20),
            None,
            mono_font::MonoTextStyleBuilder::new()
                .font(&FONT)
                .text_color(Rgb565::BLACK)
                .build(),
            text::TextStyleBuilder::new()
                .baseline(text::Baseline::Top)
                .build(),
        ),
        "Hello world!"
    )
    .unwrap();

    delay.delay_ms(2000);

    // Do nothing
    panic!("Panic demonstration");
}
