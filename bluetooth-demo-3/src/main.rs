#![no_std]
#![no_main]

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
    gpio::{Input, Pull},
    prelude::*,
    rng::Rng,
    time,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::ble::controller::BleConnector;
use rng::RngWrapper;

extern crate alloc;

const GATT_HID_SERVICE_UUID: u16 = 0x1812;
const GATT_BATTERY_SERVICE_UUID: u16 = 0x180f;

struct KeyboardReport {
    modifiers: u8,
    reserved: u8,
    key_codes: [u8; 6],
}

impl KeyboardReport {
    fn to_bytes(&self) -> [u8; 8] {
        [
            self.modifiers,
            self.reserved,
            self.key_codes[0],
            self.key_codes[1],
            self.key_codes[2],
            self.key_codes[3],
            self.key_codes[4],
            self.key_codes[5],
        ]
    }
}

macro_rules! count {
    () => { 0u8 };
    ($x:tt $($xs:tt)*) => {1u8 + count!($($xs)*)};
}

macro_rules! hid {
    ($(( $($xs:tt),*)),+ $(,)?) => { &[ $( (count!($($xs)*)-1) | $($xs),* ),* ] };
}

// Main items
pub const HIDINPUT: u8 = 0x80;
pub const HIDOUTPUT: u8 = 0x90;
pub const FEATURE: u8 = 0xb0;
pub const COLLECTION: u8 = 0xa0;
pub const END_COLLECTION: u8 = 0xc0;

// Global items
pub const USAGE_PAGE: u8 = 0x04;
pub const LOGICAL_MINIMUM: u8 = 0x14;
pub const LOGICAL_MAXIMUM: u8 = 0x24;
pub const PHYSICAL_MINIMUM: u8 = 0x34;
pub const PHYSICAL_MAXIMUM: u8 = 0x44;
pub const UNIT_EXPONENT: u8 = 0x54;
pub const UNIT: u8 = 0x64;
pub const REPORT_SIZE: u8 = 0x74; //bits
pub const REPORT_ID: u8 = 0x84;
pub const REPORT_COUNT: u8 = 0x94; //bytes
pub const PUSH: u8 = 0xa4;
pub const POP: u8 = 0xb4;

// Local items
pub const USAGE: u8 = 0x08;
pub const USAGE_MINIMUM: u8 = 0x18;
pub const USAGE_MAXIMUM: u8 = 0x28;
pub const DESIGNATOR_INDEX: u8 = 0x38;
pub const DESIGNATOR_MINIMUM: u8 = 0x48;
pub const DESIGNATOR_MAXIMUM: u8 = 0x58;
pub const STRING_INDEX: u8 = 0x78;
pub const STRING_MINIMUM: u8 = 0x88;
pub const STRING_MAXIMUM: u8 = 0x98;
pub const DELIMITER: u8 = 0xa8;

const KEYBOARD_ID: u8 = 0x01;

const REPORT_MAP: &[u8] = hid!(
    (USAGE_PAGE, 0x01), // USAGE_PAGE (Generic Desktop Ctrls)
    (USAGE, 0x06),      // USAGE (Keyboard)
    (COLLECTION, 0x01), // COLLECTION (Application)
    // ------------------------------------------------- Keyboard
    (REPORT_ID, KEYBOARD_ID), //   REPORT_ID (1)
    (USAGE_PAGE, 0x07),       //   USAGE_PAGE (Kbrd/Keypad)
    (USAGE_MINIMUM, 0xE0),    //   USAGE_MINIMUM (0xE0)
    (USAGE_MAXIMUM, 0xE7),    //   USAGE_MAXIMUM (0xE7)
    (LOGICAL_MINIMUM, 0x00),  //   LOGICAL_MINIMUM (0)
    (LOGICAL_MAXIMUM, 0x01),  //   Logical Maximum (1)
    (REPORT_SIZE, 0x01),      //   REPORT_SIZE (1)
    (REPORT_COUNT, 0x08),     //   REPORT_COUNT (8)
    (HIDINPUT, 0x02), //   INPUT (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    (REPORT_COUNT, 0x01), //   REPORT_COUNT (1) ; 1 byte (Reserved)
    (REPORT_SIZE, 0x08), //   REPORT_SIZE (8)
    (HIDINPUT, 0x01), //   INPUT (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    (REPORT_COUNT, 0x05), //   REPORT_COUNT (5) ; 5 bits (Num lock, Caps lock, Scroll lock, Compose, Kana)
    (REPORT_SIZE, 0x01),  //   REPORT_SIZE (1)
    (USAGE_PAGE, 0x08),   //   USAGE_PAGE (LEDs)
    (USAGE_MINIMUM, 0x01), //   USAGE_MINIMUM (0x01) ; Num Lock
    (USAGE_MAXIMUM, 0x05), //   USAGE_MAXIMUM (0x05) ; Kana
    (HIDOUTPUT, 0x02), //   OUTPUT (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    (REPORT_COUNT, 0x01), //   REPORT_COUNT (1) ; 3 bits (Padding)
    (REPORT_SIZE, 0x03), //   REPORT_SIZE (3)
    (HIDOUTPUT, 0x01), //   OUTPUT (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    (REPORT_COUNT, 0x06), //   REPORT_COUNT (6) ; 6 bytes (Keys)
    (REPORT_SIZE, 0x08), //   REPORT_SIZE(8)
    (LOGICAL_MINIMUM, 0x00), //   LOGICAL_MINIMUM(0)
    (LOGICAL_MAXIMUM, 0x65), //   LOGICAL_MAXIMUM(0x65) ; 101 keys
    (USAGE_PAGE, 0x07), //   USAGE_PAGE (Kbrd/Keypad)
    (USAGE_MINIMUM, 0x00), //   USAGE_MINIMUM (0)
    (USAGE_MAXIMUM, 0x65), //   USAGE_MAXIMUM (0x65)
    (HIDINPUT, 0x00),  //   INPUT (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    (END_COLLECTION),  // END_COLLECTION
);

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

    esp_alloc::heap_allocator!(72 * 1024);

    //spawner.must_spawn(ticktock());

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    log::warn!("WARN");
    log::error!("ERROR");
    log::debug!("DEBUG");
    log::trace!("TRACE");
    log::info!("INFO");

    log::info!("Press button to continue");
    let mut button = Input::new(peripherals.GPIO9, Pull::Up);
    button.wait_for_low().await;
    log::info!("Let's go!");

    let rng = Rng::new(peripherals.RNG);
    let init = esp_wifi::init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK)
        .inspect_err(|err| log::error!("Error during init, {err:?}"))
        .unwrap();

    log::info!("INIT COMPLETE");

    let mut bluetooth = peripherals.BT;

    log::info!("Retrieve bluetooth peripheral");

    let connector = BleConnector::new(&init, &mut bluetooth);

    let now = || time::now().duration_since_epoch().to_millis();
    let mut ble = Ble::new(connector, now);
    log::info!("Connector created");

    let local_ble_address = ble
        .cmd_read_br_addr()
        .await
        .expect("Failed to read local BLE address");

    log::info!("BLE address: {:02x?}", local_ble_address);

    let pin_ref = RefCell::new(button);
    let pin_ref = &pin_ref;

    loop {
        println!("{:?}", ble.init().await);
        println!("{:?}", ble.cmd_set_le_advertising_parameters().await);
        println!(
            "{:?}",
            ble.cmd_set_le_advertising_data(
                create_advertising_data(&[
                    AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                    AdStructure::ServiceUuids16(&[
                        Uuid::Uuid16(GATT_BATTERY_SERVICE_UUID),
                        Uuid::Uuid16(GATT_HID_SERVICE_UUID),
                    ]),
                    AdStructure::CompleteLocalName("El schlizos"),
                    // Use a keyboard icon
                    AdStructure::Unknown {
                        ty: 0x19,            // Appearance
                        data: &[0xc1, 0x03]  // 0x03c1 == Keyboard
                    }
                ])
                .unwrap()
            )
            .await
        );
        log::info!("{:?}", ble.cmd_set_le_advertise_enable(true).await);

        println!("started advertising");

        let desc_value = &[KEYBOARD_ID, 1];

        let mut read_hid_information = |_offset: usize, mut data: &mut [u8]| {
            log::info!("read_hid_information");
            data.write(&[0x11, 0x01, 0x00, 0x01])
                .inspect_err(|e| log::error!("read_hid_information error: {e}"))
                .unwrap_or_default()
        };

        let mut read_hid_report_map = |offset: usize, data: &mut [u8]| {
            log::info!("read_hid_report_map {offset} {}", data.len());

            let val = REPORT_MAP;
            let off = offset;
            if off < val.len() {
                let len = data.len().min(val.len() - off);
                data[..len].copy_from_slice(&val[off..off + len]);
                len
            } else {
                0
            }
        };

        let mut read_hid_report = |_offset: usize, mut data: &mut [u8]| {
            log::info!("read_hid_report");
            let resp = KeyboardReport {
                modifiers: 0,
                reserved: 0,
                key_codes: [0u8; 6],
            };

            data.write(&resp.to_bytes())
                .inspect_err(|e| log::error!("read_hid_report error: {e}"))
                .unwrap_or_default()
        };

        let mut read_protocol_mode = |_offset: usize, mut data: &mut [u8]| {
            log::info!("read_protocol_mode");
            data.write(&[0x01])
                .inspect_err(|e| log::error!("read_protocol_mode error: {e}"))
                .unwrap_or_default()
        };

        let mut write_protocol_mode = |offset: usize, data: &[u8]| {
            log::info!("write_protocol_mode: Offset {}, data {:?}", offset, data);
        };

        let mut read_device_info = |_offset: usize, mut data: &mut [u8]| {
            log::info!("read_device_info");
            data.write(&[0x02, 0x8a, 0x24, 0x66, 0x82, 0x34, 0x36])
                .inspect_err(|e| log::error!("read_device_info error: {e}"))
                .unwrap_or_default()
        };

        let mut read_battery_level = |_offset: usize, mut data: &mut [u8]| {
            log::info!("read_battery_level");
            data.write(&[100])
                .inspect_err(|e| log::error!("read_battery_level error: {e}"))
                .unwrap_or_default()
        };

        gatt!([
            service {
                //uuid: "00001812-0000-1000-8000-00805f9b34fb",
                uuid: "1812",
                characteristics: [
                    characteristic {
                        // HID information
                        //uuid: "00002a4a-0000-1000-8000-00805f9b34fb",
                        uuid: "2a4a",
                        read: read_hid_information,
                    },
                    characteristic {
                        // Report Map
                        //uuid: "2a4b-0000-1000-8000-00805f9b34fb",
                        uuid: "2a4b",
                        read: read_hid_report_map,
                    },
                    characteristic {
                        // BLE HID Report characteristic
                        //uuid: "00002a4d-0000-1000-8000-00805f9b34fb",
                        uuid: "2a4d",
                        name: "hid_report",
                        notify: true,
                        read: read_hid_report,
                        descriptors: [descriptor {
                            uuid: "2908",
                            value: desc_value,
                        }],
                    },
                    characteristic {
                        //uuid: "00002a4e-0000-1000-8000-00805f9b34fb",
                        uuid: "2a4e",
                        write: write_protocol_mode,
                        read: read_protocol_mode,
                    },
                ],
            },
            // BLE device information
            service {
                //uuid: "0000180a-0000-1000-8000-00805f9b34fb",
                uuid: "180a",
                characteristics: [characteristic {
                    // BLE Device Information characteristic
                    //uuid: "00002a50-0000-1000-8000-00805f9b34fb",
                    uuid: "2a50",
                    read: read_device_info,
                }],
            },
            // BLE HID Battery Service
            service {
                //uuid: "0000180f-0000-1000-8000-00805f9b34fb",
                uuid: "180f",
                // BLE HID battery level characteristic
                characteristics: [characteristic {
                    //uuid: "00002a19-0000-1000-8000-00805f9b34fb",
                    uuid: "2a19",
                    read: read_battery_level,
                    notify: true,
                }],
            },
        ]);

        let mut rng = RngWrapper::from(rng);
        let ltk: Option<u128> = None; // TODO: i think we need this for persistent pairing?
        let mut srv = AttributeServer::new_with_ltk(
            &mut ble,
            &mut gatt_attributes,
            Addr::from_le_bytes(false, local_ble_address),
            ltk,
            &mut rng,
        );

        let mut notifier = || {
            async {
                pin_ref.borrow_mut().wait_for_any_edge().await;
                let pressed = pin_ref.borrow_mut().is_low();

                // TODO: this should send key presses to the host, but it doesn't seem to work...
                log::info!("notify hid report (pressed = {pressed})");
                let resp = KeyboardReport {
                    modifiers: 0,
                    reserved: 0,
                    key_codes: [if pressed { 0x04 } else { 0 }, 0, 0, 0, 0, 0],
                };

                NotificationData::new(hid_report_handle, &resp.to_bytes())
            }
        };

        let r = srv.run(&mut notifier).await;
        log::error!("BLE service stopped ({r:?})");

        // DON'T REMOVE THIS
        loop {
            Timer::after(Duration::from_secs(1)).await;
        }
    }
}

#[embassy_executor::task]
async fn ticktock() {
    loop {
        log::info!("tick");
        Timer::after(Duration::from_secs(3)).await;
        log::info!("tock");
        Timer::after(Duration::from_secs(3)).await;
    }
}
