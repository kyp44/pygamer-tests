use core::fmt::Write;
use core::panic::PanicInfo;
use embedded_graphics::{mono_font, pixelcolor::Rgb565};
use embedded_graphics::{prelude::*, text};
use pygamer::hal::clock::GenericClockController;
use pygamer::hal::delay::Delay;
use pygamer::Pins;

// TODO: No crate that does this exists, should we generalize and publish it?
pub struct TextWriter<'a, D, C> {
    display: &'a mut D,
    /// Location of the text on the screen.
    position: Point,
    /// Optional box size for character wrapping.
    char_box_size: Option<Size>,
    character_style: mono_font::MonoTextStyle<'static, C>,
    text_style: text::TextStyle,
    /// Current cursor location in character space.
    char_cursor: Point,
}
impl<'a, D, C> TextWriter<'a, D, C> {
    pub fn new(
        display: &'a mut D,
        position: Point,
        box_size: Option<Size>,
        character_style: mono_font::MonoTextStyle<'static, C>,
        text_style: text::TextStyle,
    ) -> Self {
        Self {
            display,
            position,
            char_box_size: box_size.map(|s| {
                Size::new(
                    s.width / character_style.font.character_size.width,
                    s.height / character_style.font.character_size.height,
                )
            }),

            character_style,
            text_style,
            char_cursor: Point::zero(),
        }
    }
}
impl<D: DrawTarget<Color = C>, C: PixelColor> core::fmt::Write for TextWriter<'_, D, C> {
    fn write_str(&mut self, mut s: &str) -> core::fmt::Result {
        loop {
            if s.is_empty() {
                break;
            }
            if let Some(cs) = self.char_box_size
                && self.char_cursor.y >= cs.height as i32
            {
                break;
            }

            let (line_s, rem_s) = match self.char_box_size {
                Some(cs) => {
                    // Iterator of character indices in the current string
                    let mut char_idxs = s.char_indices();

                    // Advance by the number of characters left on the current line
                    let _ = char_idxs.advance_by(cs.width as usize - self.char_cursor.x as usize);

                    let idx = char_idxs.next().map(|t| t.0).unwrap_or(s.len());
                    s.split_at_checked(idx).ok_or(core::fmt::Error)?
                }
                None => (s, ""),
            };

            text::Text::with_text_style(
                line_s,
                self.position
                    + Point::new(
                        self.char_cursor.x * self.character_style.font.character_size.width as i32,
                        self.char_cursor.y * self.character_style.font.character_size.height as i32,
                    ),
                self.character_style,
                self.text_style,
            )
            .draw(self.display)
            .map_err(|_| core::fmt::Error)?;

            // Update cursor
            self.char_cursor.x += line_s.len() as i32;

            // Advance to the next line if applicable
            if let Some(cs) = self.char_box_size
                && self.char_cursor.x >= cs.width as i32
            {
                self.char_cursor.x = 0;
                self.char_cursor.y += 1;
            }

            s = rem_s;
        }

        Ok(())
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();

    // Get the peripherals
    let mut peripherals = unsafe { pygamer::pac::Peripherals::steal() };
    let core = unsafe { pygamer::pac::CorePeripherals::steal() };

    // Setup the clocks
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.gclk,
        &mut peripherals.mclk,
        &mut peripherals.osc32kctrl,
        &mut peripherals.oscctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = Pins::new(peripherals.port).split();

    // Initialize the display
    let mut delay = Delay::new(core.SYST, &mut clocks);
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

    let mut w = TextWriter::new(
        &mut display,
        Point::zero(),
        Some(crate::DISPLAY_SIZE),
        mono_font::MonoTextStyleBuilder::new()
            .font(&crate::FONT)
            .text_color(Rgb565::BLACK)
            .background_color(Rgb565::RED)
            .build(),
        text::TextStyleBuilder::new()
            .alignment(text::Alignment::Left)
            .baseline(text::Baseline::Top)
            .build(),
    );

    let _ = write!(w, "{}", info);

    loop {
        cortex_m::asm::wfi();
    }
}
