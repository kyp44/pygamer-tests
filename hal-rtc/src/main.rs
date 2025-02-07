#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use core::fmt::Write;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text;
use hal::ehal::delay::DelayNs;
use hal::rtc::{Count32Mode, Rtc};
use nb::block;
use pac::{CorePeripherals, Peripherals};
use pygamer::{entry, ButtonReader, DisplayDriver};
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

fn wait_for_button<'a>(
    display: &'a mut DisplayDriver,
    reader: &mut ButtonReader,
) -> DisplayWriter<'a, DisplayDriver, Rgb565> {
    text::Text::with_text_style(
        "Press a button to continue...",
        Point::new(0, (DISPLAY_SIZE.height - 1) as i32),
        TEXT_STYLE,
        text::TextStyleBuilder::new()
            .baseline(text::Baseline::Bottom)
            .build(),
    )
    .draw(display)
    .unwrap();

    reader.wait_for_button();
    display.clear(BACKGROUND_COLOR).unwrap();

    DisplayWriter::new(display, &DISPLAY_TEXT_STYLE)
}

fn get_counts(rtc: &Rtc<Count32Mode>) -> [u32; NUM_SAMPLES] {
    let mut counts = [0; NUM_SAMPLES];

    counts[0] = rtc.count32();
    for i in 1..NUM_SAMPLES {
        counts[i] = wait_for_count_change(&rtc);
    }

    counts
}

#[entry]
fn main() -> ! {
    let mut pkg = setup(
        Peripherals::take().unwrap(),
        CorePeripherals::take().unwrap(),
    );

    // TODO: This is broken and panics, looking at the code, this makes total sense
    // so that the code is not written correctly.
    /* writeln!(writer, "Starting RTC timer for {DELAY_SECS} seconds").unwrap();
    rtc.start(DELAY_SECS.secs());
    block!(rtc.wait());
    writeln!(writer, "Timer complete!").unwrap(); */

    let mut rtc_per = Some(pkg.rtc);

    loop {
        let mut rtc = Rtc::count32_mode(rtc_per.take().unwrap(), 32768u32.Hz(), &mut pkg.mclk);

        // Counter value sequence
        let mut writer = DisplayWriter::new(&mut pkg.display, &DISPLAY_TEXT_STYLE);
        writeln!(writer, "Basic counter test: {:?}", get_counts(&rtc)).unwrap();

        // Set counter test
        let mut writer = wait_for_button(&mut pkg.display, &mut pkg.button_reader);
        rtc.set_count32(1_000);
        writeln!(writer, "Set counter test: {:?}", get_counts(&rtc)).unwrap();

        // TODO Next test!
        let mut writer = wait_for_button(&mut pkg.display, &mut pkg.button_reader);
        rtc.reset_and_compute_prescaler(1.hours());
        writeln!(writer, "Reset with prescalar: {:?}", get_counts(&rtc)).unwrap();

        rtc_per = Some(rtc.free());
    }
}
