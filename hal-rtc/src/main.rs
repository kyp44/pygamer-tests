#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{mono_font, text};
use hal::ehal::delay::DelayNs;
use hal::rtc::{Count32Mode, Rtc};
use nb::block;
use pac::{CorePeripherals, Peripherals};
use pygamer::entry;
use shared::prelude::*;

const DELAY_SECS: u32 = 3;
const NUM_SAMPLES: usize = 50;

fn wait_for_count_change(rtc: &Rtc<Count32Mode>) -> u32 {
    let mut last_count = rtc.count32();

    loop {
        let count = rtc.count32();

        if count != last_count {
            break count;
        }

        last_count = count;
    }
}

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

    let mut rtc = Rtc::count32_mode(pkg.rtc, 1u32.kHz(), &mut pkg.mclk);

    // TODO: This is broken and panics, looking at the code, this makes total sense
    // so that the code is not written correctly.
    /* writeln!(writer, "Starting RTC timer for {DELAY_SECS} seconds").unwrap();
    rtc.start(DELAY_SECS.secs());
    block!(rtc.wait());
    writeln!(writer, "Timer complete!").unwrap(); */

    // Show the count sequence
    writeln!(writer, "RTC count values:").unwrap();
    let mut counts = [0; NUM_SAMPLES];

    // TODO: This is also busted in that `count32()` always returns 0.
    // This is likely because COUNTSYNC is not set, which testing indicates
    // will result in the inability to read any COUNT values.

    // The the array with monotonic counts
    counts[0] = rtc.count32();
    for i in 1..NUM_SAMPLES {
        counts[i] = wait_for_count_change(&rtc);
    }

    writeln!(writer, "{counts:?}");

    loop {
        cortex_m::asm::wfi();
    }
}
