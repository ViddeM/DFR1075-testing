#![no_std]
#![no_main]
// required by embassy-executor nightly feature
#![feature(impl_trait_in_assoc_type)]

use bt_hci::controller::ExternalController;
use embassy_executor::Spawner;
use embassy_futures::select::select3;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{prelude::*, rng::Rng, timer::timg::TimerGroup};
use esp_println as _;
use esp_wifi::ble::controller::BleConnector;
use trouble_host::{
    gap::{appearance, GapConfig, PeripheralConfig},
    prelude::*,
    Address, Controller, HostResources, PacketQos,
};

/// Size of L2CAP packets (ATT MTU is this - 4)
const L2CAP_MTU: usize = 251;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

type Resources<C> = HostResources<C, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

#[gatt_server(attribute_data_size = 10)]
struct MyFirstBTEServer {
    derp: Derp,
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

    let (stack, bt_peripheral, _, mut runner) = host.set_random_address(address).build();

    let server = MyFirstBTEServer::new_with_config(
        stack,
        GapConfig::Peripheral(PeripheralConfig {
            name: "Tux sux",
            appearance: &appearance::GENERIC_UNKNOWN,
        }),
    );

    defmt::info!("Starting advertising business making people by scamming people that needs medicine! #LoveCorporateAmerica");
    let res = select3(
        runner.run(),
        gatt_task(&server),
        advertise_task(bt_peripheral, &server),
    )
    .await;

    defmt::info!("res: {:?}", defmt::Debug2Format(&res));

    defmt::warn!("No idea, tux told me to do it");
}

async fn gatt_task<C: Controller>(server: &MyFirstBTEServer<'_, '_, C>) {
    loop {
        match server.next().await {
            Ok(GattEvent::Write {
                handle,
                connection: _,
            }) => {
                let _ = server.get(handle, |value| {
                    defmt::info!(
                        "[gatt] Write event on {:?}. Value written: {:?}",
                        handle,
                        value
                    );
                });
            }
            Ok(GattEvent::Read {
                handle,
                connection: _,
            }) => {
                defmt::info!("[gatt] Read event on {:?}", handle);
            }
            Err(e) => {
                defmt::error!("[gatt] Error processing GATT events: {:?}", e);
            }
        }
    }
}

struct HidInformation;

#[gatt_service(uuid = "1812")]
struct Keyboard {
    #[characteristic(uuid = "2a4a", read)]
    hid_information: HidInformation,
}

#[gatt_service(uuid = "180f")]
struct Derp {
    #[characteristic(uuid = "2a19", read, notify)]
    level: u8,
}

async fn advertise_task<C: Controller>(
    mut peripheral: Peripheral<'_, C>,
    server: &MyFirstBTEServer<'_, '_, C>,
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
        let mut advertiser = peripheral
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
        // Keep connection alive
        let mut tick: u8 = 0;
        while conn.is_connected() {
            Timer::after(Duration::from_secs(2)).await;
            tick = tick.wrapping_add(1);
            defmt::info!("[adv] notifying connection of tick {}", tick);
            let _ = server.notify(server.derp.level, &conn, &[tick]).await;
        }
    }
}
