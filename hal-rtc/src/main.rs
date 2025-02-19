#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use core::fmt::Write;
use derive_new::new;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text;
use hal::ehal::delay::DelayNs;
use hal::rtc::{Count32Mode, Rtc};
use nb::block;
use pac::{CorePeripherals, Peripherals};
use pygamer::{entry, ButtonReader, DisplayDriver};
use shared::prelude::_embedded_hal_timer_CountDown as Countdown;
use shared::prelude::*;

const DELAY_SECS: u64 = 3;
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

fn get_counts(rtc: &Rtc<Count32Mode>) -> [u32; NUM_SAMPLES] {
    let mut counts = [0; NUM_SAMPLES];

    counts[0] = rtc.count32();
    for i in 1..NUM_SAMPLES {
        counts[i] = wait_for_count_change(&rtc);
    }

    counts
}

#[derive(new)]
struct ExecPackage {
    display: DisplayDriver,
    button_reader: ButtonReader,
}
impl ExecPackage {
    pub fn reset_display(&mut self) -> DisplayWriter<DisplayDriver, Rgb565> {
        self.display.clear(BACKGROUND_COLOR).unwrap();

        DisplayWriter::new(&mut self.display, &DISPLAY_TEXT_STYLE)
    }

    fn wait_for_button(&mut self) -> DisplayWriter<DisplayDriver, Rgb565> {
        text::Text::with_text_style(
            "Press a button to continue...",
            Point::new(0, (DISPLAY_SIZE.height - 1) as i32),
            TEXT_STYLE,
            text::TextStyleBuilder::new()
                .baseline(text::Baseline::Bottom)
                .build(),
        )
        .draw(&mut self.display)
        .unwrap();

        self.button_reader.wait_for_button();
        self.reset_display()
    }
}

#[entry]
fn main() -> ! {
    let mut pkg = setup(
        Peripherals::take().unwrap(),
        CorePeripherals::take().unwrap(),
    );

    let mut epkg = ExecPackage::new(pkg.display, pkg.button_reader);
    let mut rtc_count = Some(Rtc::count32_mode(pkg.rtc, 32768u32.Hz(), &mut pkg.mclk));

    loop {
        let mut rtc = rtc_count.take().unwrap();

        // Basic counter value sequence
        let mut writer = epkg.reset_display();
        writeln!(writer, "Basic counter test: {:?}", get_counts(&rtc)).unwrap();

        // Set counter test
        let mut writer = epkg.wait_for_button();
        rtc.set_count32(1_000);
        writeln!(writer, "Set counter test: {:?}", get_counts(&rtc)).unwrap();

        // Set the presclar test, which slows the tick rate by a factor of 1024
        let mut writer = epkg.wait_for_button();
        writeln!(writer, "Slowing down the tick rate...").unwrap();
        rtc.reset_and_compute_prescaler(1.hours());
        let counts = get_counts(&rtc);
        let mut writer = epkg.reset_display();
        writeln!(writer, "Reset with prescalar: {:?}", counts).unwrap();

        // Periodic countdown timer test
        let mut writer = epkg.wait_for_button();
        writeln!(
            writer,
            "Starting periodic RTC timer for {DELAY_SECS} seconds..."
        )
        .unwrap();
        rtc.enable_interrupt();
        Countdown::start(&mut rtc, DELAY_SECS.secs());
        block!(Countdown::wait(&mut rtc)).unwrap();
        writeln!(writer, "Waiting another {DELAY_SECS} seconds...").unwrap();
        block!(Countdown::wait(&mut rtc)).unwrap();
        writeln!(writer, "Just one more delay of {DELAY_SECS} seconds...").unwrap();
        block!(Countdown::wait(&mut rtc)).unwrap();
        writeln!(writer, "Timer test complete!").unwrap();

        // Now convert into clock mode
        // TODO!

        // Wait for button before looping
        epkg.wait_for_button();

        rtc_count = Some(rtc);
    }
}
