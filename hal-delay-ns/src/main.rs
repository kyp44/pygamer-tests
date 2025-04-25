//! Tests HAL items for which `embedded_hal::delay::DelayNs` is implemented.
#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]
#![feature(generic_const_exprs)]

use core::fmt::Write;
use derive_new::new;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text;
use hal::ehal::delay::DelayNs;
use hal::{rtc::Rtc, timer::TimerCounter};
use pac::{CorePeripherals, Peripherals};
use pygamer::{entry, ButtonReader, DisplayDriver};
use shared::prelude::*;

const DELAY_SECS: u32 = 3;
const DELAY_MILLIS: u32 = DELAY_SECS * 1000;

/// TODO: Should this be moved ot the shared library? Useful for a number of tests.
#[derive(new)]
struct ExecPackage {
    display: DisplayDriver,
    button_reader: ButtonReader,
}
impl ExecPackage {
    pub fn clear_display(&mut self) -> DisplayWriter<DisplayDriver, Rgb565> {
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
        self.clear_display()
    }

    fn test_delay<D: DelayNs>(&mut self, name: &str, delay: &mut D) {
        let mut writer = self.clear_display();

        writeln!(writer, "Delaying {DELAY_SECS} for `{name}`...").unwrap();
        delay.delay_ms(DELAY_MILLIS);
        writeln!(writer, "Waiting another {DELAY_SECS} seconds...").unwrap();
        delay.delay_ms(DELAY_MILLIS);
        writeln!(writer, "One more delay of {DELAY_SECS} seconds...").unwrap();
        delay.delay_ms(DELAY_MILLIS);
        writeln!(writer, "\n`DelayNs` test for `{name}` complete!").unwrap();
        self.wait_for_button();
    }
}

#[entry]
fn main() -> ! {
    let mut pkg = setup(
        Peripherals::take().unwrap(),
        CorePeripherals::take().unwrap(),
    );

    // Setup up the execution package
    let mut exec = ExecPackage::new(pkg.display, pkg.button_reader);

    // Setup the timer
    let gclk = pkg.clocks.gclk0();
    let tc4_tc5_clock = pkg.clocks.tc4_tc5(&gclk).unwrap();
    let mut timer = TimerCounter::tc4_(&tc4_tc5_clock, pkg.tc4, &mut pkg.mclk);

    // Setup the RTC
    let mut rtc = Rtc::count32_mode(pkg.rtc, RTC_CLOCK_RATE, &mut pkg.mclk);

    let mut writer = exec.clear_display();
    writeln!(writer, "Press A to start the tests").unwrap();
    exec.wait_for_button();

    loop {
        exec.test_delay("Delay", &mut pkg.delay);
        exec.test_delay("TimerCounter", &mut timer);
        //exec.test_delay("Rtc", &mut rtc);
    }
}
