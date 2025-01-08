use atsamd_hal::{rtc::rtic::rtc_clock, rtc_monotonic};
use core::fmt::Write;
use core::future::Future;
use embedded_graphics::prelude::*;
use embedded_graphics::text;
use pygamer::hal::prelude::*;
use pygamer::{pac, DisplayDriver};

use crate::display::{DisplayTextStyle, DisplayWriter};
use crate::DISPLAY_TEXT_STYLE;
use crate::FONT;

#[cfg(not(any(feature = "clock1k", feature = "clock32k")))]
compile_error!("A clock feature must be specified");

#[cfg(feature = "clock1k")]
type _ClockRate = rtc_clock::Clock1k;
#[cfg(feature = "clock32k")]
type _ClockRate = rtc_clock::Clock32k;

#[cfg(feature = "systick")]
rtic_monotonics::systick_monotonic!(Mono, 200);
#[cfg(not(feature = "systick"))]
rtc_monotonic!(Mono, _ClockRate);

#[cfg(any(feature = "systick"))]
impl Mono {
    #[inline]
    pub fn delay_ms(delay: u32) -> impl Future<Output = ()> {
        Self::delay(delay.millis())
    }

    #[cfg(feature = "debug")]
    #[inline]
    pub fn delay_ms_debug(delay: u32, id: u32) -> impl Future<Output = ()> {
        Self::delay_debug(delay.millis(), id)
    }
}

#[cfg(not(any(feature = "systick")))]
impl Mono {
    #[inline]
    pub fn delay_ms(delay: u32) -> impl Future<Output = ()> {
        Self::delay(u64::from(delay).millis())
    }

    #[cfg(feature = "debug")]
    #[inline]
    pub fn delay_ms_debug(delay: u32, id: u32) -> impl Future<Output = ()> {
        Self::delay_debug(u64::from(delay).millis(), id)
    }
}

impl Mono {
    pub fn general_start(_syst: pac::SYST, _rtc: pac::Rtc) {
        #[cfg(feature = "systick")]
        Mono::start(_syst, 120_000_000);
        #[cfg(not(feature = "systick"))]
        Mono::start(_rtc);
    }
}

pub fn display_monotonic_info(display: &mut DisplayDriver) {
    let style = DisplayTextStyle::new(
        Point::new(160 - FONT.character_size.width as i32 * 9, 128),
        None,
        DISPLAY_TEXT_STYLE,
        text::TextStyleBuilder::new()
            .baseline(text::Baseline::Bottom)
            .build(),
    );

    let mut writer = DisplayWriter::new(display, &style);

    #[cfg(feature = "systick")]
    write!(writer, "systick  ").unwrap();
    #[cfg(not(feature = "systick"))]
    write!(writer, "mode0 ").unwrap();

    #[cfg(feature = "clock1k")]
    write!(writer, " 1k").unwrap();
    #[cfg(feature = "clock32k")]
    write!(writer, "32k").unwrap();
}
