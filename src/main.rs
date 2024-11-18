#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use core::fmt::Write;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{mono_font, text};
use pygamer::hal::clock::GenericClockController;
use pygamer::hal::delay::Delay;
use pygamer::hal::prelude::*;
use pygamer::{DisplayDriver, Pins};

mod panic_handler;

static DISPLAY_SIZE: Size = Size::new(160, 128);
const FONT: mono_font::MonoFont = mono_font::ascii::FONT_5X8;

pub use panic_handler::TextWriter;
use rtic::Mutex;

rtic_monotonics::systick_monotonic!(Mono, 20);

async fn test_task<D: Mutex<T = DisplayDriver>>(
    mut display: D,
    task_num: u8,
    cycle_time_ms: u32,
) -> ! {
    let mut cycles: usize = 0;

    loop {
        display.lock(|d| {
            write!(
                TextWriter::new(
                    d,
                    Point::new(10, 20 * task_num as i32),
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

        Mono::delay(cycle_time_ms.millis()).await;

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
        Mono::start(delay.free(), 120_000_000);

        test_1::spawn().ok().unwrap();
        test_2::spawn().ok().unwrap();
        test_3::spawn().ok().unwrap();
        test_panic::spawn().ok().unwrap();

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
        test_task(cx.shared.display, 1, 250).await
    }

    #[task(priority = 1, shared=[display])]
    async fn test_2(cx: test_2::Context) {
        test_task(cx.shared.display, 2, 500).await
    }

    #[task(priority = 1, shared=[display])]
    async fn test_3(cx: test_3::Context) {
        test_task(cx.shared.display, 3, 750).await
    }

    #[task(priority = 1)]
    async fn test_panic(_cx: test_panic::Context) {
        Mono::delay(10u32.secs()).await;
        panic!("Panic demonstration");
    }
}
