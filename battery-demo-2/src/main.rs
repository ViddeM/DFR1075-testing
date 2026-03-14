#![no_std]
#![no_main]

use core::cmp::{max, min};

use embassy_executor::Spawner;
use embassy_time::Duration;
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcCalCurve, AdcCalScheme, AdcConfig, Attenuation},
    clock::CpuClock,
    gpio::{Input, InputConfig, Io, Pin, Pull},
    interrupt::software::SoftwareInterruptControl,
    peripherals::{self, ADC1, GPIO, GPIO0},
    timer::timg::TimerGroup,
};

const FOUR_THOUSAND_NINETY_FIVE: u16 = 4095;

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let mut peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    log::warn!("WARN");
    log::error!("ERROR");
    log::debug!("DEBUG");
    log::trace!("TRACE");
    log::info!("INFO");

    log::info!("Press button to continue");
    let mut button = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );
    button.wait_for_low().await;
    log::info!("Let's go!");

    loop {
        match sample(&mut peripherals.GPIO0, &mut peripherals.ADC1) {
            Ok(voltage) => {
                log::info!("Voltage {voltage}");
            }
            Err(err) => {
                log::error!("Oh noes, failed to sample battery level {err}");
            }
        }

        embassy_time::block_for(Duration::from_millis(200));
    }
}

fn sample(pin: &mut GPIO0<'static>, adc: &mut ADC1) -> Result<f32, &'static str> {
    let mut read_atten = |attenuation: Attenuation| -> Result<(u16, f32), &'static str> {
        let mut adc_config = AdcConfig::new();
        // adc_config.enable_pin(pin.reborrow(), Attenuation::_11dB);
        let mut pin =
            adc_config.enable_pin_with_cal::<_, AdcCalCurve<ADC1>>(pin.reborrow(), attenuation);

        let mut adc = Adc::new(adc.reborrow(), adc_config);
        let raw = adc.read_oneshot(&mut pin);

        let raw = match raw {
            Ok(r) => r,
            Err(err) => {
                log::error!("AWHMAHGAHD failed to read battery level {err:?}");
                return Err("failed to read battery level");
            }
        };

        // TODO: adc_cali_raw_to_voltage, what is this shit???
        let voltage_mv = AdcCalCurve::<ADC1>::new_cal(attenuation).adc_val(raw);
        let voltage = (voltage_mv as f32) / 1000.0;

        // let voltage = ((raw as f32) * 3.3) / 4095.0;

        Ok((raw, voltage))
    };

    let (raw11, mv11) = match read_atten(Attenuation::_11dB) {
        Ok(v) => v,
        Err(err) => {
            log::error!("Failed to read attenuation for 11DB");
            return Err(err);
        }
    };

    let mut raw6 = 4095;
    let mut raw2 = 4095;
    let mut raw0 = 4095;
    let mut mv6 = 0.0;
    let mut mv2 = 0.0;
    let mut mv0 = 0.0;

    if raw11 < 4095 {
        match read_atten(Attenuation::_6dB) {
            Ok((r6, m6)) => {
                raw6 = r6;
                mv6 = m6;
            }
            Err(err) => {
                log::error!("Failed to read attenuation for 6DB");
                return Err(err);
            }
        };
    }

    if raw6 < 4095 {
        match read_atten(Attenuation::_2p5dB) {
            Ok((r2, m2)) => {
                raw2 = r2;
                mv2 = m2;
            }
            Err(err) => {
                log::error!("Failed to read attenuation for 2.5DB");
                return Err(err);
            }
        };
    }

    if raw2 < 4095 {
        match read_atten(Attenuation::_0dB) {
            Ok((r0, m0)) => {
                raw0 = r0;
                mv0 = m0;
            }
            Err(err) => {
                log::error!("Failed to read attenuation for 2.5DB");
                return Err(err);
            }
        };
    }

    // TODO: Check that we haven't missed any signages where important!
    let adc_half = 2048u16;
    let c11 = min(raw11, adc_half) as u32;
    let c6_signed = (adc_half as i32) - (raw6.abs_diff(adc_half) as i32);
    let c6 = max(c6_signed, 0) as u32;
    let c2_signed = (adc_half as i32) - (raw2.abs_diff(adc_half) as i32);
    let c2 = max(c2_signed, 0) as u32;
    let c0 = min(4095 - raw0, adc_half) as u32;

    let sum = c11 + c6 + c2 + c0;

    log::info!("Lol, I was told to print stuff");

    log::info!("Autorange summary:");
    log::info!("  Raw readings: 11db={raw11}, 6db={raw6}, 2.5db={raw2}, 0db={raw0}");
    log::info!("  Voltages: 11db={mv11:.6}, 6db={mv6:.6}, 2.5db={mv2:.6}, 0db={mv0:.6}");
    log::info!("  Coefficients: c11={c11}, c6={c6}, c2={c2}, c0={c0}, sum={sum}");

    if sum == 0 {
        log::error!("Invalid weight sum in autorange calculation");
        return Err("Invalid weight sum in autorange calculation");
    }

    let final_result =
        (mv11 * c11 as f32 + mv6 * c6 as f32 + mv2 * c2 as f32 + mv0 * c0 as f32) / sum as f32;

    log::info!("Autorange final: {final_result:.6}V");
    Ok(final_result)
}
