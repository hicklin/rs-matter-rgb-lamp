//! The example implements an RGB Light device.
#![no_std]
#![no_main]
#![recursion_limit = "256"]

use core::pin::pin;
use embassy_futures::select::{select, Either};

use alloc::boxed::Box;

use embassy_executor::Spawner;

use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;

use log::info;

use rs_matter_embassy::epoch::epoch;

use rs_matter_embassy::matter::dm::clusters::desc::{self, ClusterHandler as _};
// use rs_matter_embassy::matter::dm::clusters::on_off::test::TestOnOffDeviceLogic;
// use rs_matter_embassy::matter::dm::clusters::on_off::{self, OnOffHooks};
use rs_matter_embassy::matter::dm::devices::test::{TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET};
use rs_matter_embassy::matter::dm::{Async, Dataver, EmptyHandler, Endpoint, EpClMatcher, Node, DeviceType};

use rs_matter_embassy::matter::utils::init::InitMaybeUninit;
use rs_matter_embassy::matter::{clusters, devices};
use rs_matter_embassy::rand::esp::{esp_init_rand, esp_rand};
use rs_matter_embassy::stack::persist::DummyKvBlobStore;
use rs_matter_embassy::wireless::esp::EspWifiDriver;
use rs_matter_embassy::wireless::{EmbassyWifi, EmbassyWifiMatterStack};

use matter_rgb_lamp::led::led;
use matter_rgb_lamp::data_model::on_off::{self, ClusterAsyncHandler as _};
use matter_rgb_lamp::data_model::level_control::{self, ClusterHandler as _};
use matter_rgb_lamp::data_model::color_control::{self, ClusterHandler as _, ColorControlHandler};

use matter_rgb_lamp::led::led_handler::LedHandler;

extern crate alloc;

const BUMP_SIZE: usize = 16500;

#[cfg(feature = "esp32")]
const HEAP_SIZE: usize = 40 * 1024; // 40KB for ESP32, which has a disjoint heap
#[cfg(any(feature = "esp32c3", feature = "esp32h2"))]
const HEAP_SIZE: usize = 160 * 1024;
#[cfg(not(any(feature = "esp32", feature = "esp32c3", feature = "esp32h2")))]
const HEAP_SIZE: usize = 186 * 1024;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_s: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    info!("Starting...");

    // Heap strictly necessary only for Wifi+BLE and for the only Matter dependency which needs (~4KB) alloc - `x509`
    // However since `esp32` specifically has a disjoint heap which causes bss size troubles, it is easier
    // to allocate the statics once from heap as well
    heap_allocator!(size: HEAP_SIZE);
    #[cfg(feature = "esp32")]
    heap_allocator!(#[link_section = ".dram2_uninit"] size: 96 * 1024);

    // == Step 1: ==
    // Necessary `esp-hal` and `esp-wifi` initialization boilerplate

    let peripherals = esp_hal::init(esp_hal::Config::default());

    // To erase generics, `Matter` takes a rand `fn` rather than a trait or a closure,
    // so we need to initialize the global `rand` fn once
    esp_init_rand(esp_hal::rng::Rng::new());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(
        timg0.timer0,
        #[cfg(target_arch = "riscv32")]
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT)
            .software_interrupt0,
    );

    let init = esp_radio::init().unwrap();

    // == Step 2: ==
    // Allocate the Matter stack.
    // For MCUs, it is best to allocate it statically, so as to avoid program stack blowups (its memory footprint is ~ 35 to 50KB).
    // It is also (currently) a mandatory requirement when the wireless stack variation is used.
    let stack =
        &*Box::leak(Box::new_uninit()).init_with(EmbassyWifiMatterStack::<BUMP_SIZE, ()>::init(
            &TEST_DEV_DET,
            TEST_DEV_COMM,
            &TEST_DEV_ATT,
            epoch,
            esp_rand,
        ));

    // == Step 3: ==
    // Set up Matter data model handler
    let channel = Channel::<CriticalSectionRawMutex, led::ControlMessage, 4>::new();
    let sender = channel.sender();

    let led_handler = LedHandler::new(sender.clone());
    let color_control_handler = ColorControlHandler::new(sender);

    // Chain our endpoint clusters
    let handler = EmptyHandler
        // Our on-off cluster, on Endpoint 1
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(on_off::OnOffCluster::<LedHandler>::CLUSTER.id),
            ),
            on_off::OnOffCluster::new(Dataver::new_rand(stack.matter().rand()), &led_handler).adapt(),
        )
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(level_control::LevelControlCluster::<LedHandler>::CLUSTER.id),
            ),
            Async(level_control::LevelControlCluster::new(Dataver::new_rand(stack.matter().rand()), &led_handler).adapt()),
        )
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(color_control::ColorControlCluster::<ColorControlHandler>::CLUSTER.id),
            ),
            Async(color_control::ColorControlCluster::new(Dataver::new_rand(stack.matter().rand()), color_control_handler).adapt()),
        )
        // Each Endpoint needs a Descriptor cluster too
        // Just use the one that `rs-matter` provides out of the box
        .chain(
            EpClMatcher::new(Some(LIGHT_ENDPOINT_ID), Some(desc::DescHandler::CLUSTER.id)),
            Async(desc::DescHandler::new(Dataver::new_rand(stack.matter().rand())).adapt()),
        );

    // == Step 4: ==
    // Run the Matter stack with our handler
    // Using `pin!` is completely optional, but reduces the size of the final future

    // Create the persister & load any previously saved state
    // `EmbassyPersist`+`EmbassyKvBlobStore` saves to a user-supplied NOR Flash region
    // However, for this demo and for simplicity, we use a dummy persister that does nothing
    let persist = stack
        .create_persist_with_comm_window(DummyKvBlobStore)
        .await
        .unwrap();

    // This step can be repeated in that the stack can be stopped and started multiple times, as needed.
    let mut matter = pin!(stack.run_coex(
        // The Matter stack needs to instantiate an `embassy-net` `Driver` and `Controller`
        EmbassyWifi::new(
            EspWifiDriver::new(&init, peripherals.WIFI, peripherals.BT),
            stack
        ),
        // The Matter stack needs a persister to store its state
        &persist,
        // Our `AsyncHandler` + `AsyncMetadata` impl
        (NODE, handler),
        // User future to run; the LED task
        (),
    ));

    // == Step 5: ==
    // Setup the LED driver
    let receiver = channel.receiver();
    let led_driver = led::Driver::new(peripherals.RMT, peripherals.GPIO8.into(), receiver);
    let mut led_task = pin!(led_driver.run());

    // == Step 6: ==
    // Run async tasks
    match select(&mut matter, &mut led_task).await {
        Either::First(_) => {
            panic!("Matter thread exited!")
        },
        Either::Second(_) => {
            panic!("LED thread exited!")
        },
    }
}

/// Endpoint 0 (the root endpoint) always runs
/// the hidden Matter system clusters, so we pick ID=1
const LIGHT_ENDPOINT_ID: u16 = 1;

const DEV_TYPE_ENHANCED_COLOR_LIGHT: DeviceType = DeviceType {
    dtype: 0x010D,
    drev: 4,
};

/// The Matter Light device Node
const NODE: Node = Node {
    id: 0,
    endpoints: &[
        EmbassyWifiMatterStack::<0, ()>::root_endpoint(),
        Endpoint {
            id: LIGHT_ENDPOINT_ID,
            device_types: devices!(DEV_TYPE_ENHANCED_COLOR_LIGHT),
            clusters: clusters!(
                desc::DescHandler::CLUSTER,
                on_off::OnOffCluster::<LedHandler>::CLUSTER,
                level_control::LevelControlCluster::<LedHandler>::CLUSTER
                color_control::ColorControlCluster::<ColorControlHandler>::CLUSTER
            ),
        },
    ],
};
