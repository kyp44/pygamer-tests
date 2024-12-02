#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{mono_font, text};
use hal::prelude::*;
use shared::prelude::*;

#[rtic::app(device = pygamer::pac, dispatchers = [EVSYS_0])]
mod app {
    use super::*;
    use hal::delay::Delay;
    use pygamer::{DisplayDriver, RedLed};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        red_led: RedLed,
        display: DisplayDriver,
        delay: Delay,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut pkg = setup(cx.device, cx.core);

        // Start the monotonic
        Mono::start(pkg.rtc, &mut pkg.mclk, &mut pkg.osc32kctrl);

        test_1::spawn().ok().unwrap();

        (
            Shared {},
            Local {
                red_led: pkg.red_led,
                display: pkg.display,
                delay: pkg.delay,
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

    #[task(priority = 1, local=[display, delay])]
    async fn test_1(cx: test_1::Context) {
        const DELAY_MS: u32 = 1000;

        let style = DisplayTextStyle::new(
            Point::new(10, 20 * 2),
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
            write!(
                DisplayWriter::new(cx.local.display, &style,),
                "Task cycles: {cycles}",
            )
            .unwrap();

            // Async delay, to add to timer queue and pend
            #[cfg(feature = "debug")]
            Mono::delay_ms_debug(DELAY_MS, 1).await;
            #[cfg(not(feature = "debug"))]
            Mono::delay_ms(DELAY_MS).await;

            // Blocking delay, during which the RTC should be disabled
            cx.local.delay.delay_ms(DELAY_MS);

            cycles = cycles.wrapping_add(1);
        }
    }
}
