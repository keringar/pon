#![feature(used)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate stm32f40x_hal as hal;

use hal::gpio::{Output, PushPull};
use hal::prelude::*;

use rtfm::{app, Resource, Threshold};

app! {
    device: hal::stm32f40x,

    resources: {
        static GPIOD12: hal::gpio::gpiod::PD12<Output<PushPull>>;
        static FLAG: bool = false;
    },

    tasks: {
        SYS_TICK: {
            path: sys_tick,
            resources: [GPIOD12, FLAG],
        }
    }
}

// Initialize peripherals
fn init(mut p: init::Peripherals, _r: init::Resources) -> init::LateResources {
    let mut rcc = p.device.RCC.constrain();
    let mut flash = p.device.FLASH.constrain();

    // Test adjusting clocks
    let clocks = rcc.cfgr
        .source(hal::rcc::ClockSource::HSI)
        .apb1_prescale(2)
        .enable_pll(96, 4, 4)
        .build(&mut flash.acr);

    assert_eq!(clocks.hclk(), 48.mhz().into());
    assert_eq!(clocks.pclk1(), 24.mhz().into());
    assert_eq!(clocks.pclk2(), 48.mhz().into());
    assert_eq!(clocks.sysclk(), 48.mhz().into());

    let mut gpiod = p.device.GPIOD.split(&mut rcc.ahb1);

    let mut pin_d12 = gpiod.pd12.into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);

    pin_d12.set_high();

    p.core.SYST.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    p.core.SYST.set_reload(clocks.sysclk().0);
    p.core.SYST.clear_current();
    p.core.SYST.enable_interrupt();
    p.core.SYST.enable_counter();

    init::LateResources {
        GPIOD12: pin_d12,
    }
}

/// Should blink at 1 hertz?
fn sys_tick(t: &mut Threshold, mut r: SYS_TICK::Resources) {
    if *r.FLAG {
        r.GPIOD12.set_low();
    } else {
        r.GPIOD12.set_high();
    }

    *r.FLAG = !*r.FLAG;
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}
