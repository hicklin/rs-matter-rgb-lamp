//! An example utilizing the `EmbassyWifiMatterStack` struct.
//!
//! As the name suggests, this Matter stack assembly uses Wifi as the main transport,
//! and thus BLE for commissioning.
//!
//! If you want to use Ethernet, utilize `EmbassyEthMatterStack` instead.
//! If you want to use concurrent commissioning, call `run_coex` instead of `run`.
//! (Note: Alexa does not work (yet) with non-concurrent commissioning.)
//!
//! The example implements a fictitious Light device (an On-Off Matter cluster).
#![no_std]
#![no_main]
#![recursion_limit = "256"]

use core::mem::MaybeUninit;
use core::pin::pin;

use alloc::boxed::Box;

use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use esp_backtrace as _;
use esp_hal::{timer::timg::TimerGroup};

use log::{info, error};

use rs_matter_embassy::epoch::epoch;
use rs_matter_embassy::matter::data_model::objects::{
    Async, Dataver, EmptyHandler, Endpoint, EpClMatcher, Node, DeviceType
};
use rs_matter_embassy::matter::data_model::system_model::desc::{self, ClusterHandler as _};
use rs_matter_embassy::matter::utils::init::InitMaybeUninit;
use rs_matter_embassy::matter::{clusters, devices};
use rs_matter_embassy::rand::esp::{esp_init_rand, esp_rand};
use rs_matter_embassy::stack::MdnsType;
use rs_matter_embassy::stack::matter::test_device::{TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET};
use rs_matter_embassy::stack::persist::DummyKvBlobStore;
use rs_matter_embassy::wireless::esp::EspWifiDriver;
use rs_matter_embassy::wireless::{EmbassyWifi, EmbassyWifiMatterStack};

use matter_rgb_lamp::led::led;
use matter_rgb_lamp::data_model::on_off::{self, ClusterAsyncHandler as _, OnOffHandler,};
use matter_rgb_lamp::data_model::level_control::{self, ClusterHandler as _, LevelControlHandler};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_s: Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Info);

    info!("Starting...");

    // Heap strictly necessary only for Wifi+BLE and for the only Matter dependency which needs (~4KB) alloc - `x509`
    // However since `esp32` specifically has a disjoint heap which causes bss size troubles, it is easier
    // to allocate the statics once from heap as well
    init_heap();

    // == Step 1: ==
    // Necessary `esp-hal` and `esp-wifi` initialization boilerplate

    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let rng = esp_hal::rng::Rng::new(peripherals.RNG);

    // To erase generics, `Matter` takes a rand `fn` rather than a trait or a closure,
    // so we need to initialize the global `rand` fn once
    esp_init_rand(rng);

    let init = esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK).unwrap();

    #[cfg(not(feature = "esp32"))]
    {
        esp_hal_embassy::init(
            esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER).alarm0,
        );
    }
    #[cfg(feature = "esp32")]
    {
        esp_hal_embassy::init(timg0.timer1);
    }

    // == Step 2: ==
    // Allocate the Matter stack.
    // For MCUs, it is best to allocate it statically, so as to avoid program stack blowups (its memory footprint is ~ 35 to 50KB).
    // It is also (currently) a mandatory requirement when the wireless stack variation is used.
    let stack = &*Box::leak(Box::new_uninit()).init_with(EmbassyWifiMatterStack::<()>::init(
        &TEST_DEV_DET,
        TEST_DEV_COMM,
        &TEST_DEV_ATT,
        MdnsType::Builtin,
        epoch,
        esp_rand,
    ));

    // == Step 3: ==
    // Set up Matter data model handler
    let channel = Channel::<CriticalSectionRawMutex, led::ControlMessage, 4>::new();
    let sender = channel.sender();

    let on_off_handler = OnOffHandler::new(sender.clone());
    let level_control_handler = LevelControlHandler::new(sender);

    // Chain our endpoint clusters
    let handler = EmptyHandler
        // Our on-off cluster, on Endpoint 1
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(on_off::OnOffCluster::<OnOffHandler>::CLUSTER.id),
            ),
            on_off::OnOffCluster::new(Dataver::new_rand(stack.matter().rand()), on_off_handler).adapt(),
        )
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(level_control::LevelControlCluster::<LevelControlHandler>::CLUSTER.id),
            ),
            Async(level_control::LevelControlCluster::new(Dataver::new_rand(stack.matter().rand()), level_control_handler).adapt()),
        )
        // Each Endpoint needs a Descriptor cluster too
        // Just use the one that `rs-matter` provides out of the box
        .chain(
            EpClMatcher::new(Some(LIGHT_ENDPOINT_ID), Some(desc::DescHandler::CLUSTER.id)),
            Async(desc::DescHandler::new(Dataver::new_rand(stack.matter().rand())).adapt()),
        );

    // == Step 4: ==
    // Run the Matter stack with our handler
    // Using `pin!` is completely optional, but saves some memory due to `rustc`
    // not being very intelligent w.r.t. stack usage in async functions
    //
    // This step can be repeated in that the stack can be stopped and started multiple times, as needed.
    let store = stack.create_shared_store(DummyKvBlobStore);
    let mut matter = pin!(stack.run(
        // The Matter stack needs to instantiate an `embassy-net` `Driver` and `Controller`
        EmbassyWifi::new(
            EspWifiDriver::new(&init, peripherals.WIFI, peripherals.BT),
            stack
        ),
        // The Matter stack needs a persister to store its state
        &store,
        // Our `AsyncHandler` + `AsyncMetadata` impl
        (NODE, handler),
        // No user future to run
        (),
    ));

    let receiver = channel.receiver();

    // == Step 5: ==
    // Setup the LED
    let led_driver = led::Driver::new(peripherals.RMT, peripherals.GPIO8.into(), receiver);
    let mut led_task = pin!(led_driver.run());

    
    // // Just for demoing purposes:
    // //
    // // Run a sample loop that simulates state changes triggered by the HAL
    // // Changes will be properly communicated to the Matter controllers
    // // (i.e. Google Home, Alexa) and other Matter devices thanks to subscriptions
    // let mut device = pin!(async {
    //     loop {
    //         // Simulate user toggling the light with a physical switch every 5 seconds
    //         Timer::after(Duration::from_secs(5)).await;

    //         // Toggle
    //         on_off.set(!on_off.get());

    //         // Let the Matter stack know that we have changed
    //         // the state of our Light device
    //         stack.notify_changed();

    //         info!("Light toggled");

    //     }
    // });

    // Schedule the Matter run & the device loop together
    // select3(&mut matter, &mut device, &mut led_task).coalesce().await.unwrap();
    match select(&mut matter, &mut led_task).await {
        Either::First(_) => {
            error!("Matter thread exited!")
        },
        Either::Second(_) => {
            error!("LED thread exited!")
        },
    }
}

/// Endpoint 0 (the root endpoint) always runs
/// the hidden Matter system clusters, so we pick ID=1
const LIGHT_ENDPOINT_ID: u16 = 1;

// todo Using this is causing home assistant to loose the device after it is added. Check that the mandatory clusters etc. are supported.
const DEV_TYPE_DIMMABLE_LIGHT: DeviceType = DeviceType {
    dtype: 0x0101,
    drev: 1,
};

/// The Matter Light device Node
const NODE: Node = Node {
    id: 0,
    endpoints: &[
        EmbassyWifiMatterStack::<()>::root_endpoint(),
        Endpoint {
            id: LIGHT_ENDPOINT_ID,
            device_types: devices!(DEV_TYPE_DIMMABLE_LIGHT),
            clusters: clusters!(
                desc::DescHandler::CLUSTER,
                on_off::OnOffCluster::<OnOffHandler>::CLUSTER,
                level_control::LevelControlCluster::<LevelControlHandler>::CLUSTER
            ),
        },
    ],
};

#[allow(static_mut_refs)]
fn init_heap() {
    fn add_region<const N: usize>(region: &'static mut MaybeUninit<[u8; N]>) {
        unsafe {
            esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
                region.as_mut_ptr() as *mut u8,
                N,
                esp_alloc::MemoryCapability::Internal.into(),
            ));
        }
    }

    #[cfg(feature = "esp32")]
    {
        // The esp32 has two disjoint memory regions for heap
        // Also, it has 64KB reserved for the BT stack in the first region, so we can't use that

        static mut HEAP1: MaybeUninit<[u8; 30 * 1024]> = MaybeUninit::uninit();
        #[link_section = ".dram2_uninit"]
        static mut HEAP2: MaybeUninit<[u8; 96 * 1024]> = MaybeUninit::uninit();

        add_region(unsafe { &mut HEAP1 });
        add_region(unsafe { &mut HEAP2 });
    }

    #[cfg(not(feature = "esp32"))]
    {
        static mut HEAP: MaybeUninit<[u8; 186 * 1024]> = MaybeUninit::uninit();

        add_region(unsafe { &mut HEAP });
    }
}
