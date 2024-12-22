#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{mono_font, text};
use hal::prelude::*;
use pygamer::pins::DisplayDriver;
use rtic::Mutex;
use shared::prelude::*;

const BASE_PERIOD_MS: u32 = 250;

async fn test_task<D: Mutex<T = DisplayDriver>>(
    mut display: D,
    task_num: u8,
    cycle_time_ms: u32,
    task_id: Option<u32>,
) -> ! {
    let style = DisplayTextStyle::new(
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
    );
    let mut cycles: usize = 0;

    loop {
        display.lock(|d| {
            write!(
                DisplayWriter::new(d, &style,),
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
        let mut pkg = setup(cx.device, cx.core);

        // Start the monotonic
        Mono::general_start(pkg.delay.free(), pkg.rtc);

        // Display selected monotonic and clock
        display_monotonic_info(&mut pkg.display);

        test_1::spawn().ok().unwrap();
        test_2::spawn().ok().unwrap();
        test_3::spawn().ok().unwrap();
        test_4::spawn().ok().unwrap();

        (
            Shared {
                display: pkg.display,
            },
            Local {
                red_led: pkg.red_led,
            },
        )
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
        test_task(cx.shared.display, 1, BASE_PERIOD_MS, Some(1)).await
    }

    #[task(priority = 1, shared=[display])]
    async fn test_2(cx: test_2::Context) {
        test_task(cx.shared.display, 2, BASE_PERIOD_MS * 2, Some(2)).await
    }

    #[task(priority = 1, shared=[display])]
    async fn test_3(cx: test_3::Context) {
        test_task(cx.shared.display, 3, BASE_PERIOD_MS * 3, Some(3)).await
    }

    #[task(priority = 1, shared=[display])]
    async fn test_4(cx: test_4::Context) {
        test_task(cx.shared.display, 4, BASE_PERIOD_MS * 4, Some(4)).await;
    }
}
