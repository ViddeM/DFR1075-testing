#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    system::SystemControl,
    timer::{timg::TimerGroup, PeriodicTimer},
};
use esp_wifi::{ble::controller::BleConnector, initialize, EspWifiInitFor};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let p_timer = PeriodicTimer::new(esp_hal::timer::ErasedTimer::Timg0Timer0(timg0.timer0));

    let init = initialize(
        EspWifiInitFor::Ble,
        p_timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let mut bluetooth = peripherals.BT;
    let connector = BleConnector::new(&init, &mut bluetooth);

    let mut ble = Ble::new(connector, esp_wifi::current_millis());
    log::info!("BLE connector created");

    loop {
        log::info!("Waiting...");
        delay.delay(1000.millis());
    }
}
