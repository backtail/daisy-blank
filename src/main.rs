#![no_main]
#![no_std]

pub mod daisy_pod;

pub mod encoder;
pub mod rgbled;
pub mod sd_card;

pub const CONTROL_RATE_IN_MS: u32 = 100;

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
        phase: f32,
        pitch: f32,
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
                phase: 0.0,
                pitch: 440.0,
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
    #[task(binds = DMA1_STR1, local = [ar, phase, pitch], shared = [], priority = 8)]
    fn audio_handler(ctx: audio_handler::Context) {
        let audio = &mut ctx.local.ar.audio;
        let buffer = &mut ctx.local.ar.buffer;
        let phase = ctx.local.phase;
        let pitch = ctx.local.pitch;

        audio.get_stereo(buffer);
        for _ in 0..buffer.len() {
            // phase is gonna get bigger and bigger
            // at some point floating point errors will quantize the pitch
            *phase += *pitch / libdaisy::AUDIO_SAMPLE_RATE as f32;
            let mono = libm::sinf(*phase);
            audio.push_stereo((mono, mono)).unwrap();

            if *pitch > 10_000.0 {
                *pitch = 440.0;
            }

            *pitch += 0.1;
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
