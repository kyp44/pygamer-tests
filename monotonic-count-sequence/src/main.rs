#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use embedded_graphics::mono_font;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text;
use hal::prelude::*;
use shared::prelude::*;

const NUM_SAMPLES: usize = 100;

fn wait_for_count_change() -> <Mono as Monotonic>::Instant {
    let mut last_count = Mono::now();

    loop {
        let count = Mono::now();

        if count != last_count {
            break count;
        }

        last_count = count;
    }
}

#[rtic::app(device = pygamer::pac, dispatchers = [EVSYS_0])]
mod app {
    use super::*;
    use pygamer::RedLed;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        red_led: RedLed,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut pkg = setup(cx.device, cx.core);

        // Start the monotonic
        Mono::general_start(
            pkg.delay.free(),
            pkg.rtc,
            &mut pkg.mclk,
            &mut pkg.osc32kctrl,
        );

        // Show the count sequence
        let mut counts = [0; NUM_SAMPLES];

        // The the array with monotonic counts
        counts[0] = Mono::now().ticks();
        for i in 1..NUM_SAMPLES {
            counts[i] = wait_for_count_change().ticks();
        }

        write!(
            DisplayWriter::new(
                &mut pkg.display,
                &DisplayTextStyle::new(
                    Point::zero(),
                    Some(DISPLAY_SIZE),
                    mono_font::MonoTextStyleBuilder::new()
                        .font(&FONT)
                        .text_color(Rgb565::BLACK)
                        .background_color(Rgb565::WHITE)
                        .build(),
                    text::TextStyleBuilder::new()
                        .baseline(text::Baseline::Top)
                        .build(),
                )
            ),
            "{counts:?}"
        )
        .unwrap();

        (
            Shared {},
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
}
