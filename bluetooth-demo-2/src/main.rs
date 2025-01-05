#![no_std]
#![no_main]
// required by embassy-executor nightly feature
#![feature(impl_trait_in_assoc_type)]

use bt_hci::controller::ExternalController;
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{prelude::*, rng::Rng, timer::timg::TimerGroup};
use esp_println as _;
use esp_wifi::ble::controller::BleConnector;
use futures::{select_biased, FutureExt};
use trouble_host::{
    gap::{GapConfig, PeripheralConfig},
    prelude::*,
    Address, Controller, HostResources, PacketQos,
};

/// Size of L2CAP packets (ATT MTU is this - 4)
// NOTE: must be AT LEAST 1017
const L2CAP_MTU: usize = 1017;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

type Resources<C> = HostResources<C, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

#[gatt_server]
struct MyFirstBTEServer {
    battery: BatteryService,
}

#[esp_hal_embassy::main]
async fn main(_s: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max(); // TODO: Maybe no max?
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);

    defmt::warn!("WARN");
    defmt::error!("ERROR");
    defmt::debug!("DEBUG");
    defmt::trace!("TRACE");
    defmt::info!("INFO");

    defmt::info!("Let's go!");

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    defmt::info!("Initializing esp_wifi");
    let init = esp_wifi::init(
        timg0.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .expect("BAIL (hilfe!)");
    defmt::info!("...done");

    // TODO: Not sure why the second one tbh.
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    let bluetooth = peripherals.BT;
    let connector = BleConnector::new(&init, bluetooth);

    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    let mut host_resources = Resources::new(PacketQos::None);
    let host = trouble_host::new(controller, &mut host_resources);

    let address = Address::random([0x41, 0x5A, 0xE3, 0x1E, 0x83, 0xE7]);
    defmt::info!(
        "I don't know, this is some address we have... I guess: `{:02x}`",
        address
    );

    let (_stack, bt_peripheral, _, mut runner) = host.set_random_address(address).build();

    let server = MyFirstBTEServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: "Tux sux",
        appearance: &appearance::HUMIDIFIER, // TODO: keyboard
    }))
    .expect("Failed to create BTE server");

    defmt::info!("Starting advertising business making people by scamming people that needs medicine! #LoveCorporateAmerica");
    select_biased! {
        r = runner.run().fuse() => {
            defmt::error!("BLE runner exited with {}", defmt::Debug2Format(&r));
        }
        r = advertise_task(bt_peripheral, &server).fuse() => {
            defmt::error!("advertise_task exited with {}", defmt::Debug2Format(&r));
        }
    }

    defmt::warn!("No idea, tux told me to do it");
}

//struct HidInformation;
//
//#[gatt_service(uuid = "1812")]
//struct Keyboard {
//    #[characteristic(uuid = "2a4a", read)]
//    hid_information: HidInformation,
//}

// Battery service
#[gatt_service(uuid = "180f")]
struct BatteryService {
    /// Battery Level
    #[descriptor(uuid = "2b20", read, value = "Battery Level")]
    #[descriptor(uuid = "2b21", read, value = [0x12, 0x34])]
    #[characteristic(uuid = "2a19", read, notify, value = 10)]
    level: u8,
}

async fn advertise_task<C: Controller>(
    mut peripheral: Peripheral<'_, C>,
    server: &MyFirstBTEServer<'_>,
) -> Result<(), BleHostError<C::Error>> {
    let mut adv_data = [0; 31];
    AdStructure::encode_slice(
        &[
            AdStructure::Flags(
                LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED | AD_FLAG_LE_LIMITED_DISCOVERABLE,
            ),
            AdStructure::ServiceUuids16(&[Uuid::Uuid16([0x0f, 0x18])]),
            AdStructure::CompleteLocalName(b"Your mom's a hoe!"),
        ],
        &mut adv_data[..],
    )?;

    loop {
        defmt::info!("[adv] advertising");
        let advertiser = peripheral
            .advertise(
                &Default::default(),
                Advertisement::ConnectableScannableUndirected {
                    adv_data: &adv_data[..],
                    scan_data: b"Hejsan",
                },
            )
            .await?;
        let conn = advertiser.accept().await?;

        defmt::info!("[adv] connection established");
        select_biased! {
            _ = conn_task(server, &conn).fuse() => {}
            _ = counter_task(server, &conn).fuse() => {}
        }
        defmt::info!("[adv] connection dropped");
    }
}

async fn conn_task(server: &MyFirstBTEServer<'_>, conn: &Connection<'_>) {
    loop {
        match conn.next().await {
            ConnectionEvent::Disconnected { reason } => {
                defmt::info!("[gatt] disconnected: {:?}", reason);
                break;
            }
            ConnectionEvent::Gatt { data } => match data.process(server).await {
                Ok(Some(GattEvent::Read(event))) => {
                    if event.handle() == server.battery.level.handle {
                        let value = server.get(&server.battery.level);
                        defmt::info!("[gatt] Read Event to Level Characteristic: {:?}", value);
                    }
                }
                Ok(Some(GattEvent::Write(event))) => {
                    if event.handle() == server.battery.level.handle {
                        defmt::info!(
                            "[gatt] Write Event to battery level characteristic: {}",
                            event.data()
                        );
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    defmt::warn!("[gatt] error processing event: {:?}", e);
                }
            },
        }
    }
}

/// Example task to use the BLE notifier interface.
async fn counter_task(server: &MyFirstBTEServer<'_>, conn: &Connection<'_>) {
    let mut tick: u8 = 0;
    let level = server.battery.level;
    loop {
        tick = tick.wrapping_add(1);
        defmt::info!("[adv] notifying connection of tick {}", tick);
        if level.notify(server, conn, &tick).await.is_err() {
            defmt::info!("[adv] error notifying connection");
            break;
        };
        Timer::after_secs(2).await;
    }
}
