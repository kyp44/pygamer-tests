#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(iter_advance_by)]

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::{mono_font, text};
use pac::{CorePeripherals, Peripherals};
use pygamer::entry;
use shared::prelude::*;

#[entry]
fn main() -> ! {
    let mut pkg = setup(
        Peripherals::take().unwrap(),
        CorePeripherals::take().unwrap(),
    );

    write!(
        DisplayWriter::new(
            &mut pkg.display,
            &DisplayTextStyle::new(
                Point::new(10, 20),
                None,
                mono_font::MonoTextStyleBuilder::new()
                    .font(&FONT)
                    .text_color(Rgb565::BLACK)
                    .build(),
                text::TextStyleBuilder::new()
                    .baseline(text::Baseline::Top)
                    .build(),
            ),
        ),
        "Hello world!"
    )
    .unwrap();

    pkg.delay.delay_ms(2000u16);

    // Do nothing
    panic!("Panic demonstration");
}
