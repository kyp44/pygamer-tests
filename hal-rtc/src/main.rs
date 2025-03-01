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
use hal::{
    ehal::delay::DelayNs,
    rtc::{ClockMode, Count32Mode, Datetime, Rtc},
};
use nb::block;
use pac::{CorePeripherals, Peripherals};
use pygamer::{entry, ButtonReader, DisplayDriver};
use shared::prelude::_embedded_hal_timer_CountDown as Countdown;
use shared::prelude::*;

const DELAY_SECS: u64 = 3;

trait RtcExt: Sized {
    type Count: Copy + Eq + Default + core::fmt::Debug;
    const NUM_SAMPLES: usize;

    fn count(&self) -> Self::Count;

    fn wait_for_count_change(&self) -> Self::Count {
        let mut last_count = self.count();

        loop {
            let count = self.count();

            if count != last_count {
                break count;
            }

            last_count = count;
        }
    }

    fn get_counts(&self) -> [Self::Count; Self::NUM_SAMPLES] {
        let mut counts = [Default::default(); Self::NUM_SAMPLES];
        counts[0] = self.count();
        for i in 1..Self::NUM_SAMPLES {
            counts[i] = self.wait_for_count_change();
        }

        counts
    }
}

impl RtcExt for Rtc<Count32Mode> {
    type Count = u32;
    const NUM_SAMPLES: usize = 10;

    fn count(&self) -> Self::Count {
        self.count32()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct FormattedDatetime(pub Datetime);
impl core::fmt::Debug for FormattedDatetime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "20{:02}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.0.year, self.0.month, self.0.day, self.0.hours, self.0.minutes, self.0.seconds
        )
    }
}

impl RtcExt for Rtc<ClockMode> {
    type Count = FormattedDatetime;
    const NUM_SAMPLES: usize = 5;

    fn count(&self) -> Self::Count {
        FormattedDatetime(self.current_time())
    }
}

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

    fn show_counts<R: RtcExt>(
        &mut self,
        rtc: &mut R,
        msg: &str,
    ) -> DisplayWriter<DisplayDriver, Rgb565>
    where
        [(); R::NUM_SAMPLES]:,
    {
        write!(self.clear_display(), "Gathering counter samples...").unwrap();
        let counts = rtc.get_counts();
        let mut writer = self.clear_display();
        writeln!(writer, "{msg}:").unwrap();
        for count in counts {
            writeln!(writer, "{:?}", count).unwrap();
        }

        self.wait_for_button()
    }

    fn date_rollover(
        &mut self,
        rtc: &mut Rtc<ClockMode>,
        year: u8,
        month: u8,
        day: u8,
        msg: &str,
    ) -> DisplayWriter<DisplayDriver, Rgb565> {
        rtc.set_time(Datetime {
            seconds: 57,
            minutes: 59,
            hours: 23,
            day,
            month,
            year,
        });
        self.show_counts(rtc, msg)
    }
}

#[entry]
fn main() -> ! {
    let mut pkg = setup(
        Peripherals::take().unwrap(),
        CorePeripherals::take().unwrap(),
    );

    let mut exec = ExecPackage::new(pkg.display, pkg.button_reader);
    let mut rtc_count = Some(Rtc::count32_mode(pkg.rtc, RTC_CLOCK_RATE, &mut pkg.mclk));

    loop {
        let mut rtc = rtc_count.take().unwrap();

        // Count 32 tests
        let _ = exec.show_counts(&mut rtc, "Basic counter test");
        rtc.set_count32(1_000);
        let _ = exec.show_counts(&mut rtc, "Set counter test");
        rtc.reset_and_compute_prescaler(1.hours());
        let mut writer = exec.show_counts(&mut rtc, "Reset with prescalar");

        // Periodic countdown timer test with the ehal 0.2 `Countdown` trait
        writeln!(
            writer,
            "Starting periodic RTC timer for {DELAY_SECS} seconds..."
        )
        .unwrap();
        Countdown::start(&mut rtc, DELAY_SECS.secs());
        block!(Countdown::wait(&mut rtc)).unwrap();
        writeln!(writer, "Waiting another {DELAY_SECS} seconds...").unwrap();
        block!(Countdown::wait(&mut rtc)).unwrap();
        writeln!(writer, "One more delay of {DELAY_SECS} seconds...").unwrap();
        block!(Countdown::wait(&mut rtc)).unwrap();
        writeln!(writer, "Periodic `Countdown` test complete!").unwrap();

        // Delay with the ehal `DelayNs` trait
        let mut writer = exec.wait_for_button();
        writeln!(writer, "Delaying for {DELAY_SECS} seconds...").unwrap();
        rtc.delay_ms(DELAY_SECS as u32 * 1000);
        writeln!(writer, "Delaying another {DELAY_SECS} seconds...").unwrap();
        rtc.delay_ms(DELAY_SECS as u32 * 1000);
        writeln!(writer, "`DelayNs` test complete!").unwrap();
        let _ = exec.wait_for_button();

        // Now test clock mode
        let mut rtc = rtc.into_clock_mode();
        exec.show_counts(&mut rtc, "Basic clock mode test");
        exec.date_rollover(&mut rtc, 24, 2, 28, "Happy leap day");
        exec.date_rollover(&mut rtc, 25, 2, 28, "No leap day here");
        exec.date_rollover(&mut rtc, 29, 12, 31, "Happy new year");

        rtc_count = Some(rtc.into_count32_mode());
    }
}
