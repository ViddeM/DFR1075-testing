#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation, Resolution},
    clock::ClockControl,
    delay::Delay,
    gpio::{Io, Output},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut output = Output::new(io.pins.gpio15, esp_hal::gpio::Level::Low);

    let analog_pin = io.pins.gpio0;

    let mut adc1_config = AdcConfig::new();

    let mut voltage_pin = adc1_config.enable_pin(analog_pin, Attenuation::Attenuation11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    let battv_max = 4.1;
    let battv_min = 3.2;
    let battv_low = 3.4;

    loop {
        let pin_value: u16 = nb::block!(adc1.read_oneshot(&mut voltage_pin)).unwrap();
        let battv = ((pin_value as f32) / 4095.0) * 3.3 * 2.0 * 1.05;
        let battery_percentage = ((battv - battv_min) / (battv_max - battv_min)) * 100.0;

        log::info!("Battery level is currently {battery_percentage}%, glhf figuring out what the actual fuck that means.");

        delay.delay(500.millis());

        output.toggle();

        delay.delay(2000.millis());
        output.toggle();
    }
}
