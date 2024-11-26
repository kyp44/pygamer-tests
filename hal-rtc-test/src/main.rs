#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use cortex_m::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{mono_font, text};
use hal::rtc::Rtc;
use nb::block;
use pac::{CorePeripherals, Peripherals};
use pygamer::entry;
use shared::prelude::*;

const DELAY_SECS: u32 = 5;

#[entry]
fn main() -> ! {
    let mut pkg = setup(
        Peripherals::take().unwrap(),
        CorePeripherals::take().unwrap(),
    );

    let style = DisplayTextStyle::new(
        Point::zero(),
        Some(DISPLAY_SIZE),
        mono_font::MonoTextStyleBuilder::new()
            .font(&FONT)
            .text_color(Rgb565::BLACK)
            .build(),
        text::TextStyleBuilder::new()
            .baseline(text::Baseline::Top)
            .build(),
    );

    let mut writer = DisplayWriter::new(&mut pkg.display, &style);

    writeln!(writer, "Starting RTC timer for {DELAY_SECS} seconds...").unwrap();

    let mut rtc = Rtc::count32_mode(pkg.rtc, 1u32.kHz(), &mut pkg.mclk);
    rtc.start(DELAY_SECS.secs());
    block!(rtc.wait()).unwrap();

    writeln!(writer, "Timer complete!").unwrap();

    loop {
        cortex_m::asm::wfi();
    }
}
