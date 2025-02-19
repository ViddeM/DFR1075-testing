#![no_std]
#![no_main]
#![allow(unreachable_code)]

mod nice_view;
mod rng;

use core::cell::RefCell;

use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    async_attribute_server::AttributeServer,
    asynch::Ble,
    attribute_server::NotificationData,
    gatt, Addr,
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_io::Write;
use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaTxBuf},
    gpio::{Input, Level, Output, Pull},
    prelude::*,
    rng::Rng,
    spi::{self, master::Spi, SpiMode},
    time,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::ble::controller::BleConnector;
use fugit::HertzU32;
use nice_view::NiceView;
use rng::RngWrapper;

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Debug);
    // esp_println::logger::init_logger_from_env();

    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    use esp_hal::timer::systimer::{SystemTimer, Target};
    let systimer = SystemTimer::new(peripherals.SYSTIMER).split::<Target>();
    esp_hal_embassy::init(systimer.alarm0);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let dma = Dma::new(peripherals.DMA);

    esp_alloc::heap_allocator!(72 * 1024);

    log::warn!("WARN");
    log::error!("ERROR");
    log::debug!("DEBUG");
    log::trace!("TRACE");
    log::info!("INFO");

    log::info!("Press button to continue");
    let mut button = Input::new(peripherals.GPIO9, Pull::Up);
    button.wait_for_low().await;
    log::info!("Let's go!");

    // 22, 21,  20
    // CS, SCK, MOSI
    let mut spi_config = spi::master::Config::default();
    spi_config.frequency = HertzU32::MHz(1);
    spi_config.mode = SpiMode::Mode0;

    // NiceView cs is active high. I think...
    let cs = Output::new(peripherals.GPIO22, Level::Low);
    let spi = Spi::new_with_config(peripherals.SPI2, spi_config)
        .with_sck(peripherals.GPIO21)
        .with_mosi(peripherals.GPIO20)
        //.with_cs(peripherals.GPIO22)
        .into_async()
        .with_dma(
            dma.channel0
                .configure(false, esp_hal::dma::DmaPriority::Priority0),
        );

    let mut nice_view = NiceView::new(spi, cs);

    log::info!("wait...");
    Timer::after(Duration::from_secs(2)).await;

    log::info!("drawing...");
    nice_view.clear_display().await;
    nice_view.testy().await;

    log::info!("exit");

    loop {}
}
