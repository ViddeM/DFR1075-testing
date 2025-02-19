#![no_std]
#![no_main]
#![allow(unreachable_code)]

mod nice_view;
mod rng;

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    dma::Dma,
    gpio::{Input, Level, Output, Pull},
    prelude::*,
    spi::{self, master::Spi, SpiBitOrder, SpiMode},
};
use fugit::HertzU32;
use nice_view::{Color, NiceView};

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
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

    // Required SPI config for the NiceView
    spi_config.frequency = HertzU32::MHz(1);
    spi_config.mode = SpiMode::Mode0;
    spi_config.write_bit_order = SpiBitOrder::LSBFirst;
    spi_config.read_bit_order = SpiBitOrder::LSBFirst; // probably useless

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

    log::info!("BOUNCE DA BALL");
    nice_view.clear_display().await;

    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut dx: f32 = 0.6;
    let mut dy: f32 = 0.0;
    let gravity = 0.01;

    let r = 8;
    nice_view.fill_white();
    loop {
        nice_view.draw_circle(x as usize, y as usize, Color::White, r);

        dy += gravity;

        x += dx;
        y += dy;

        if x >= 160.0 && dx > 0.0 {
            dx = -dx;
        }
        if x < 0.0 && dx < 0.0 {
            dx = -dx;
        }

        if y >= 68.0 && dy > 0.0 {
            dy = -dy;
        }
        if y < 0.0 && dy < 0.0 {
            dy = -dy;
        }

        nice_view.draw_circle(x as usize, y as usize, Color::Black, r);
        nice_view.flush().await;

        //Timer::after(Duration::from_millis(10)).await;
    }

    log::info!("exit");

    loop {}
}
