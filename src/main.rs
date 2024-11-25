#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use core::fmt::Write;
use core::future::Future;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{mono_font, text};
use pygamer::hal::clock::GenericClockController;
use pygamer::hal::delay::Delay;
use pygamer::hal::prelude::*;
use pygamer::{pins::DisplayDriver, Pins};
use rtic::Mutex;

mod panic_handler;

static DISPLAY_SIZE: Size = Size::new(160, 128);
const FONT: mono_font::MonoFont = mono_font::ascii::FONT_5X8;

pub use panic_handler::TextWriter;

#[cfg(not(any(feature = "clock1k", feature = "clock32k")))]
compile_error!("A clock feature must be specified");

#[cfg(not(any(feature = "systick", feature = "mode0", feature = "mode1")))]
compile_error!("Monotonic mode feature must be specified");

#[cfg(feature = "clock1k")]
type _ClockRate = rtc_clock::Clock1k;
#[cfg(feature = "clock32k")]
type _ClockRate = rtc_clock::Clock32k;

#[cfg(feature = "systick")]
rtic_monotonics::systick_monotonic!(Mono, 200);
#[cfg(feature = "mode0")]
rtc_mode0_monotonic!(Mono, _ClockRate, rtc_clock::ClockInternal);
#[cfg(feature = "mode1")]
rtc_mode1_monotonic!(Mono, _ClockRate, rtc_clock::ClockInternal);

#[cfg(any(feature = "systick", feature = "mode0"))]
impl Mono {
    #[inline]
    fn delay_ms(delay: u32) -> impl Future<Output = ()> {
        Self::delay(delay.millis())
    }

    #[cfg(feature = "debug")]
    #[inline]
    fn delay_ms_debug(delay: u32, id: u8) -> impl Future<Output = ()> {
        Self::delay_debug(delay.millis(), id)
    }
}

#[cfg(feature = "mode1")]
impl Mono {
    #[inline]
    fn delay_ms(delay: u32) -> impl Future<Output = ()> {
        Self::delay(u64::from(delay).millis())
    }

    #[cfg(feature = "debug")]
    #[inline]
    fn delay_ms_debug(delay: u32, id: u8) -> impl Future<Output = ()> {
        Self::delay_debug(u64::from(delay).millis(), id)
    }
}

async fn test_task<D: Mutex<T = DisplayDriver>>(
    mut display: D,
    task_num: u8,
    cycle_time_ms: u32,
    task_id: Option<u8>,
) -> ! {
    let mut cycles: usize = 0;

    loop {
        display.lock(|d| {
            write!(
                TextWriter::new(
                    d,
                    Point::new(10, 20 * (task_num + 1) as i32),
                    None,
                    mono_font::MonoTextStyleBuilder::new()
                        .font(&FONT)
                        .text_color(Rgb565::BLACK)
                        .background_color(Rgb565::WHITE)
                        .build(),
                    text::TextStyleBuilder::new()
                        .baseline(text::Baseline::Top)
                        .build(),
                ),
                "Task number {task_num} cycles: {cycles}",
            )
            .unwrap();
        });

        match task_id {
            Some(_id) => {
                #[cfg(feature = "debug")]
                Mono::delay_ms_debug(cycle_time_ms, _id).await;
                #[cfg(not(feature = "debug"))]
                Mono::delay_ms(cycle_time_ms).await;
            }
            None => Mono::delay_ms(cycle_time_ms).await,
        }

        cycles = cycles.wrapping_add(1);
    }
}

#[rtic::app(device = pygamer::pac, dispatchers = [EVSYS_0])]
mod app {
    use super::*;
    use pygamer::{DisplayDriver, RedLed};

    #[shared]
    struct Shared {
        display: DisplayDriver,
    }

    #[local]
    struct Local {
        red_led: RedLed,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Get the peripherals
        let mut peripherals = cx.device;
        let core = cx.core;

        // Setup clocks
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

        // Set up the red LED
        let red_led = pins.led_pin.into();

        // Start the monotonic
        #[cfg(feature = "systick")]
        Mono::start(delay.free(), 120_000_000);
        #[cfg(any(feature = "mode0", feature = "mode1"))]
        Mono::start(
            peripherals.rtc,
            &mut peripherals.mclk,
            &mut peripherals.osc32kctrl,
        );

        test_1::spawn().ok().unwrap();
        test_2::spawn().ok().unwrap();
        test_3::spawn().ok().unwrap();
        test_late::spawn().ok().unwrap();

        (Shared { display }, Local { red_led })
    }

    #[idle(local = [red_led])]
    fn idle(cx: idle::Context) -> ! {
        loop {
            rtic::export::wfi();
            cx.local.red_led.toggle().unwrap();
        }
    }

    #[task(priority = 1, shared=[display])]
    async fn test_1(cx: test_1::Context) {
        test_task(cx.shared.display, 1, 250, Some(1)).await
    }

    #[task(priority = 1, shared=[display])]
    async fn test_2(cx: test_2::Context) {
        test_task(cx.shared.display, 2, 500, Some(2)).await
    }

    #[task(priority = 1, shared=[display])]
    async fn test_3(cx: test_3::Context) {
        test_task(cx.shared.display, 3, 750, Some(3)).await
    }

    #[task(priority = 1, shared=[display])]
    async fn test_late(cx: test_late::Context) {
        Mono::delay_ms(10_000).await;
        test_task(cx.shared.display, 4, 1000, Some(4)).await;
    }
}
