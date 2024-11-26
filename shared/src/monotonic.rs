use core::future::Future;
use pygamer::hal::prelude::*;
use pygamer::pac;

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
    pub fn delay_ms(delay: u32) -> impl Future<Output = ()> {
        Self::delay(delay.millis())
    }

    #[cfg(feature = "debug")]
    #[inline]
    pub fn delay_ms_debug(delay: u32, id: u8) -> impl Future<Output = ()> {
        Self::delay_debug(delay.millis(), id)
    }
}

#[cfg(feature = "mode1")]
impl Mono {
    #[inline]
    pub fn delay_ms(delay: u32) -> impl Future<Output = ()> {
        Self::delay(u64::from(delay).millis())
    }

    #[cfg(feature = "debug")]
    #[inline]
    pub fn delay_ms_debug(delay: u32, id: u8) -> impl Future<Output = ()> {
        Self::delay_debug(u64::from(delay).millis(), id)
    }
}

impl Mono {
    pub fn general_start(
        _syst: pac::SYST,
        _rtc: pac::Rtc,
        _mclk: &mut pac::Mclk,
        _osc32kctrl: &mut pac::Osc32kctrl,
    ) {
        #[cfg(feature = "systick")]
        Mono::start(_syst, 120_000_000);
        #[cfg(any(feature = "mode0", feature = "mode1"))]
        Mono::start(_rtc, _mclk, _osc32kctrl);
    }
}
