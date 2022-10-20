#![no_main]
#![no_std]

pub mod daisy_pod;

pub mod encoder;
pub mod rgbled;
pub mod sd_card;

pub const CONTROL_RATE_IN_MS: u32 = 10;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
)]
mod app {
    use crate::daisy_pod::{AudioRate, ControlRate, DaisyPod};
    use libdaisy::prelude::*;

    #[cfg(feature = "log")]
    use rtt_target::rprintln;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        ar: AudioRate,
        cr: ControlRate,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // initiate system
        let daisy = DaisyPod::init(ctx.core, ctx.device);

        libdaisy::logger::init();

        #[cfg(feature = "log")]
        {
            // init logging via RTT
            info!("RTT loggging initiated!");
        }

        (
            Shared {},
            Local {
                ar: daisy.audio_rate,
                cr: daisy.control_rate,
            },
            init::Monotonics(),
        )
    }

    // Non-default idle ensures chip doesn't go to sleep which causes issues for
    // probe.rs currently
    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    // Interrupt handler for audio
    #[task(binds = DMA1_STR1, local = [ar], shared = [], priority = 8)]
    fn audio_handler(ctx: audio_handler::Context) {
        let audio = &mut ctx.local.ar.audio;
        let mut buffer = ctx.local.ar.buffer;

        audio.get_stereo(&mut buffer);

        for (left, right) in buffer {
            audio.push_stereo((left, right)).unwrap();
        }
    }

    #[task(binds = TIM2, local = [cr], shared = [])]
    fn update_handler(ctx: update_handler::Context) {
        // clear TIM2 interrupt flag
        ctx.local.cr.timer2.clear_irq();

        // get all hardware
        let adc1 = &mut ctx.local.cr.adc1;
        let pot1 = &mut ctx.local.cr.pot1;
        let pot2 = &mut ctx.local.cr.pot2;
        let switch1 = &mut ctx.local.cr.switch1;
        let switch2 = &mut ctx.local.cr.switch2;
        let led1 = &mut ctx.local.cr.led1;
        let led2 = &mut ctx.local.cr.led2;
        let encoder = &mut ctx.local.cr.encoder;

        // update all the hardware
        if let Ok(data) = adc1.read(pot1.get_pin()) {
            pot1.update(data);
        }
        if let Ok(data) = adc1.read(pot2.get_pin()) {
            pot2.update(data);
        }
        led1.update();
        led2.update();
        switch1.update();
        switch2.update();
        encoder.update();

        // do something
    }
}
