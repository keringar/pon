#![feature(used)]
#![feature(proc_macro)]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
extern crate cortex_m_rtfm as rtfm;
#[macro_use]
extern crate stm32f40x;

use rtfm::{app, Threshold, Resource};

pub enum Direction {
    North,
    West,
    South,
    East,
}

app! {
    device: stm32f40x,

    resources: {
        static DIRECTION: Direction = Direction::North;
        static GPIOD: stm32f40x::GPIOD;
    },

    tasks: {
        SYS_TICK: {
            path: sys_tick,
            resources: [DIRECTION, GPIOD],
        }
    }
}

// Initialize peripherals
fn init(mut p:init::Peripherals, _r: init::Resources) -> init::LateResources {
    // Enable gpio D
    p.device.RCC.ahb1enr.write(|w| unsafe { w.bits(0b1 << 3) });

    // configure Discovery LEDs as push-pull output
    for pin in 12..16 {
        // General purpose output mode
        let offset = pin * 2;
        let mode = 0b01;
        p.device.GPIOD
            .moder
            .modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << offset)) | (mode << offset)) });

        // Push pull output
        p.device.GPIOD
            .otyper
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << pin)) });
    }

    // Configure system timer to generate one interrupt a second
    p.core.SYST.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    p.core.SYST.set_reload(16_000_000);
    p.core.SYST.clear_current();
    p.core.SYST.enable_interrupt();
    p.core.SYST.enable_counter();

    init::LateResources {
        GPIOD: p.device.GPIOD,
    }
}

fn sys_tick(t: &mut Threshold, mut r: SYS_TICK::Resources) {
    match *r.DIRECTION {
        Direction::North => {
            // Clear East pin and enable North pin
            r.GPIOD.claim(t, |gpiod, t| {
                gpiod.bsrr.write(|w| w.br15().reset() );
                gpiod.bsrr.write(|w| w.bs12().set() );
            });

            *r.DIRECTION = Direction::West;
        },
        Direction::West => {
            // Clear North pin and enable West pin
            r.GPIOD.claim(t, |gpiod, t| {
                gpiod.bsrr.write(|w| w.br12().reset() );
                gpiod.bsrr.write(|w| w.bs13().set() );
            });

            *r.DIRECTION = Direction::South;
        },
        Direction::South => {
            // Clear West pin and enable South pin
            r.GPIOD.claim(t, |gpiod, t| {
                gpiod.bsrr.write(|w| w.br13().reset() );
                gpiod.bsrr.write(|w| w.bs14().set() );
            });

            *r.DIRECTION = Direction::East;
        },
        Direction::East => {
            // Clear South pin and enable East pin
            r.GPIOD.claim(t, |gpiod, t| {
                gpiod.bsrr.write(|w| w.br14().reset() );
                gpiod.bsrr.write(|w| w.bs15().set() );
            });

            *r.DIRECTION = Direction::North;
        },
    }
}

fn idle() -> ! {
    loop {
        rtfm::wfi();
    }
}