use embedded_graphics::mono_font;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives;
use embedded_graphics::text;
use pygamer::DisplayDriver;
use rtic::Mutex;
use shared::prelude::*;

const COLORS: &[Rgb565] = &[
    Rgb565::RED,
    Rgb565::GREEN,
    Rgb565::BLUE,
    Rgb565::CYAN,
    Rgb565::MAGENTA,
    Rgb565::YELLOW,
    Rgb565::BLACK,
    Rgb565::CSS_PINK,
    Rgb565::CSS_ORANGE,
    Rgb565::CSS_PURPLE,
    Rgb565::WHITE,
    Rgb565::CSS_GRAY,
];

pub async fn test_task<D: Mutex<T = DisplayDriver>>(
    mut display: D,
    position: u32,
    delay_ms: u32,
) -> ! {
    const PATCH_SIZE: u32 = 5;
    const ROW_LEN: u32 = 160 / PATCH_SIZE;

    let rectangle = primitives::Rectangle::new(
        Point::new(
            ((position % ROW_LEN) * PATCH_SIZE) as i32,
            ((position / ROW_LEN) * PATCH_SIZE) as i32,
        ),
        Size::new(PATCH_SIZE, PATCH_SIZE),
    );

    for color in COLORS.iter().cycle() {
        display.lock(|d| {
            rectangle
                .into_styled(primitives::PrimitiveStyle::with_fill(*color))
                .draw(d)
                .unwrap();
        });

        #[cfg(feature = "debug")]
        Mono::delay_ms_debug(delay_ms, position).await;
        #[cfg(not(feature = "debug"))]
        Mono::delay_ms(delay_ms).await;
    }

    panic!();
}

pub async fn clock_task<D: Mutex<T = DisplayDriver>>(mut display: D) -> ! {
    let style = DisplayTextStyle::new(
        Point::new(0, 128),
        None,
        mono_font::MonoTextStyleBuilder::new()
            .font(&crate::FONT)
            .text_color(Rgb565::WHITE)
            .background_color(Rgb565::BLACK)
            .build(),
        text::TextStyleBuilder::new()
            .baseline(text::Baseline::Bottom)
            .build(),
    );

    loop {
        display.lock(|d| {
            write!(DisplayWriter::new(d, &style), "0x{:X}", Mono::now().ticks()).unwrap();
        });

        Mono::delay_ms(500).await;
    }
}

pub async fn neopixels_task<S: SmartLedsWrite<Color = RGB8>>(neopixels: &mut S, delay_ms: u32) -> !
where
    S::Error: core::fmt::Debug,
{
    let mut colors = [
        RGB8::new(10, 0, 0),
        RGB8::new(0, 10, 0),
        RGB8::new(0, 0, 10),
    ]
    .into_iter()
    .cycle();

    loop {
        neopixels.write(colors.clone().take(5)).unwrap();

        colors.next().unwrap();

        Mono::delay_ms(delay_ms).await;
    }
}
