#![no_std]
#![no_main]
#![allow(unreachable_code)]

mod nice_view;

use embassy_executor::Spawner;
use embassy_time::Duration;
use esp_backtrace as _;
use esp_hal::gpio::{InputConfig, OutputConfig};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::spi::Mode;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, Level, Output, Pull},
    i2c::master::{I2c, Operation},
    spi::{self, master::Spi},
};
use nice_view::NiceView;
use niceview_lib::{Icon, KeyboardDisplay, TextVariant};

extern crate alloc;

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    esp_println::logger::init_logger(log::LevelFilter::Info);
    // esp_println::logger::init_logger_from_env();

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

    // 22, 21,  20
    // CS, SCK, MOSI
    // Required SPI config for the NiceView
    let spi_config = spi::master::Config::default()
        .with_frequency(Rate::from_mhz(1))
        .with_mode(Mode::_0)
        .with_write_bit_order(spi::BitOrder::LsbFirst)
        .with_read_bit_order(spi::BitOrder::LsbFirst); // probably useless

    // NiceView cs is active high. I think...
    let cs = Output::new(peripherals.GPIO23, Level::Low, OutputConfig::default());

    let gpio_expander_reset = Output::new(peripherals.GPIO2, Level::High, OutputConfig::default());

    let spi = Spi::new(peripherals.SPI2, spi_config)
        .expect("SPI config is valid")
        .with_sck(peripherals.GPIO22)
        .with_mosi(peripherals.GPIO21)
        .with_dma(peripherals.DMA_CH0)
        .into_async();

    let mut i2c = I2c::new(
        peripherals.I2C0.reborrow(),
        esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100)),
    )
    .expect("I2C config to be valid")
    .with_sda(peripherals.GPIO6)
    .with_scl(peripherals.GPIO7)
    .into_async();

    const ADDRESS: u8 = 0b1110100;

    log::info!("Writing to GPIO expander...");
    // Configure all pins in both ports as inputs.
    embassy_time::with_timeout(
        Duration::from_millis(200),
        i2c.write_async(
            ADDRESS,
            &[GpioECommand::ConfigurationPort0 as u8, 0xFF, 0xFF],
        ),
    )
    .await
    .expect("TIMEOUT for config reached :cry:")
    .expect("Failed to configure GPIO expander");

    embassy_time::with_timeout(
        Duration::from_millis(200),
        i2c.write_async(
            ADDRESS,
            &[GpioECommand::PolarityInversionPort0 as u8, 0xff, 0xff],
        ),
    )
    .await
    .expect("Timeout during i2c polarity conversion")
    .expect("Failed to call polarity inversion on gpio expander");

    loop {
        let mut buf: [u8; 2] = [0; 2];

        embassy_time::with_timeout(
            Duration::from_millis(200),
            i2c.transaction_async(
                ADDRESS,
                [
                    &mut Operation::Write(&[GpioECommand::InputPort0 as u8]),
                    &mut Operation::Read(&mut buf),
                ],
            ),
        )
        .await
        .expect("TIMEOUT REACHED")
        .expect("Failed write to gpio expander");

        // embassy_time::with_timeout(
        //     Duration::from_millis(200),
        //     i2c.write_read_async(ADDRESS, &[GpioECommand::InputPort0 as u8], &mut buf),
        // )
        // .await
        // .expect("TIMEOUT REACHED")
        // .expect("Failed write to gpio expander");

        // embassy_time::with_timeout(
        //     Duration::from_millis(200),
        //     i2c.write_async(ADDRESS, &[GpioECommand::InputPort0 as u8]),
        // )
        // .await
        // .expect("TIMEOUT REACHED")
        // .expect("Failed write to gpio expander");

        // embassy_time::with_timeout(
        //     Duration::from_millis(200),
        //     i2c.read_async(ADDRESS, &mut buf),
        // )
        // .await
        // .expect("TIMEOUT REACHED (2)")
        // .expect("Failed read from gpio expander");

        log::info!("Read the following: {:#08b} {:#08b}", buf[0], buf[1]);
    }

    // let mut nice_view = NiceView::new(spi, cs);

    // nice_view.clear_display().await;
    // nice_view.fill_white();

    // nice_view.draw_text("HELLO WORLD!", 10, 10, TextVariant::Regular);

    // nice_view.draw_icon(Icon::BatteryFull, 40, 40);

    // nice_view.flush().await;

    // loop {
    //     nice_view.fill_white();
    // }
}

#[repr(u8)]
#[allow(unused)]
enum GpioECommand {
    InputPort0 = 0,
    InputPort1 = 1,
    OuputPort0 = 2,
    OuputPort1 = 3,
    PolarityInversionPort0 = 4,
    PolarityInversionPort1 = 5,
    ConfigurationPort0 = 6,
    ConfigurationPort1 = 7,
}
