//! The example implements an RGB Light device.
#![no_std]
#![no_main]
#![recursion_limit = "256"]

use core::pin::pin;

use embassy_executor::Spawner;

use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::ram;
use esp_hal::timer::timg::TimerGroup;
use esp_metadata_generated::memory_range;

#[cfg(feature = "defmt")]
use defmt::info;
#[cfg(feature = "log")]
use log::info;

use rand_core::SeedableRng as _;
use rs_matter_embassy::epoch::epoch;
use rs_matter_embassy::matter::crypto::{Crypto, default_crypto};
// Data Model imports
use rs_matter_embassy::matter::dm::clusters::{
    app::level_control::{
        self, AttributeDefaults, ClusterAsyncHandler as _, LevelControlHandler, OptionsBitmap,
    },
    app::on_off::{self, ClusterAsyncHandler as _, OnOffHandler},
    desc::{self, ClusterHandler as _},
};
use rs_matter_embassy::matter::dm::devices::test::{
    DAC_PRIVKEY, TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET,
};
use rs_matter_embassy::matter::dm::{
    Async, Dataver, DeviceType, EmptyHandler, Endpoint, EpClMatcher, Node,
};

use rs_matter_embassy::matter::dm::clusters::decl::color_control::ClusterHandler as _;
use rs_matter_embassy::matter::persist::DummyKvBlobStore;
use rs_matter_embassy::matter::tlv::Nullable;
use rs_matter_embassy::matter::utils::init::InitMaybeUninit;
use rs_matter_embassy::matter::{clusters, devices};
use rs_matter_embassy::stack::rand::reseeding_csprng;
use rs_matter_embassy::wireless::esp::EspWifiDriver;
use rs_matter_embassy::wireless::{EmbassyWifi, EmbassyWifiMatterStack};

use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::gpio::{Input, InputConfig, Pull};

use embassy_futures::select::{Either, Either3, select, select3};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;

use matter_rgb_lamp::dm::color_control;
use matter_rgb_lamp::led::rgb_led_driver::{self, LedSender};
use matter_rgb_lamp::led::led_handler::LedHandler;

use static_cell::StaticCell;
// LED setup
use esp_hal::rmt::PulseCode;
use esp_hal_smartled::buffer_size_async;

extern crate alloc;

macro_rules! mk_static {
    ($t:ty) => {{
        #[cfg(not(feature = "esp32"))]
        {
            static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
            STATIC_CELL.uninit()
        }
        #[cfg(feature = "esp32")]
        alloc::boxed::Box::leak(alloc::boxed::Box::<$t>::new_uninit())
    }};
}

const NUM_LEDS: usize = 1;

/// The amount of memory for allocating all `rs-matter-stack` futures created during
/// the execution of the `run*` methods.
/// This does NOT include the rest of the Matter stack.
///
/// The futures of `rs-matter-stack` created during the execution of the `run*` methods
/// are allocated in a special way using a small bump allocator which results
/// in a much lower memory usage by those.
///
/// If - for your platform - this size is not enough, increase it until
/// the program runs without panics during the stack initialization.
const BUMP_SIZE: usize = 20000;

/// Heap strictly necessary only for Wifi+BLE and for the only Matter dependency which needs (~4KB) alloc - `x509`
#[cfg(not(feature = "esp32"))]
const HEAP_SIZE: usize = 100 * 1024;
/// On the esp32, we allocate the Matter Stack from heap as well, due to the non-contiguous memory regions on that chip
#[cfg(feature = "esp32")]
const HEAP_SIZE: usize = 140 * 1024;

const RECLAIMED_RAM: usize =
    memory_range!("DRAM2_UNINIT").end - memory_range!("DRAM2_UNINIT").start;

esp_bootloader_esp_idf::esp_app_desc!();

#[cfg(feature = "defmt")]
use esp_println as _;

#[esp_rtos::main]
async fn main(_s: Spawner) {
    #[cfg(feature = "log")]
    esp_println::logger::init_logger(log::LevelFilter::Info);

    info!("Starting...");

    heap_allocator!(size: HEAP_SIZE - RECLAIMED_RAM);
    heap_allocator!(#[ram(reclaimed)] size: RECLAIMED_RAM);

    // == Step 1: ==
    // Necessary `esp-hal` and `esp-wifi` initialization boilerplate

    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(
        timg0.timer0,
        #[cfg(target_arch = "riscv32")]
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT)
            .software_interrupt0,
    );

    // == Step 2: ==
    // Allocate the Matter stack.
    // For MCUs, it is best to allocate it statically, so as to avoid program stack blowups (its memory footprint is ~ 35 to 50KB).
    // It is also (currently) a mandatory requirement when the wireless stack variation is used.
    let stack = mk_static!(EmbassyWifiMatterStack::<BUMP_SIZE, ()>).init_with(
        EmbassyWifiMatterStack::init(&TEST_DEV_DET, TEST_DEV_COMM, &TEST_DEV_ATT, epoch),
    );

    // TODO: Change this to using Trng when it's got an API that dose not consume any peripherals that we need!
    let mut seed = [0u8; 32];
    esp_hal::rng::Rng::new().read(&mut seed);
    let crypto = default_crypto(
        reseeding_csprng(rand_chacha::ChaCha20Rng::from_seed(seed), 1000).unwrap(),
        DAC_PRIVKEY,
    );
    let mut weak_rand = crypto.weak_rand().unwrap();

    // == Step 3: ==
    // Set up Matter data model handler
    let channel = Channel::<CriticalSectionRawMutex, rgb_led_driver::ControlMessage, 4>::new();
    let sender = channel.sender();

    let button_on_off = Input::new(
        peripherals.GPIO7,
        InputConfig::default().with_pull(Pull::Up),
    );

    let mut adc1_config = AdcConfig::new();
    let pin = adc1_config.enable_pin(peripherals.GPIO4, Attenuation::_11dB);
    let adc1 = Adc::new(peripherals.ADC1, adc1_config);

    let led_handler = LedHandler::new(sender, button_on_off, adc1, pin);

    let on_off_handler = OnOffHandler::new(
        Dataver::new_rand(&mut weak_rand),
        LIGHT_ENDPOINT_ID,
        &led_handler,
    );
    let level_control_handler = LevelControlHandler::new(
        Dataver::new_rand(&mut weak_rand),
        LIGHT_ENDPOINT_ID,
        &led_handler,
        AttributeDefaults {
            on_level: Nullable::none(),
            options: OptionsBitmap::EXECUTE_IF_OFF,
            ..Default::default()
        },
    );

    on_off_handler.init(Some(&level_control_handler));
    level_control_handler.init(Some(&on_off_handler));

    // Chain our endpoint clusters
    let handler = EmptyHandler
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(OnOffHandler::<LedHandler<LedSender>, LedHandler<LedSender>>::CLUSTER.id),
            ),
            on_off::HandlerAsyncAdaptor(&on_off_handler),
        )
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(LevelControlHandler::<LedHandler<LedSender>, LedHandler<LedSender>>::CLUSTER.id),
            ),
            level_control::HandlerAsyncAdaptor(&level_control_handler),
        )
        .chain(
            EpClMatcher::new(
                Some(LIGHT_ENDPOINT_ID),
                Some(color_control::ColorControlHandler::<LedHandler<LedSender>>::CLUSTER.id),
            ),
            Async(
                color_control::ColorControlHandler::new(
                    Dataver::new_rand(&mut weak_rand),
                    &led_handler,
                )
                .adapt(),
            ),
        )
        .chain(
            EpClMatcher::new(Some(LIGHT_ENDPOINT_ID), Some(desc::DescHandler::CLUSTER.id)),
            Async(desc::DescHandler::new(Dataver::new_rand(&mut weak_rand)).adapt()),
        );

    // == Step 4: ==
    // Create a KV BLOB store and load any previously saved state of `rs-matter`
    // `SeqMapKvBlobStore` saves to a user-supplied NOR Flash region
    // However, for this demo and for simplicity, we use a dummy KV BLOB store that does nothing
    let mut kv = DummyKvBlobStore;
    stack.startup(&crypto, &mut kv).await.unwrap();

    // Wrap the KV BLOB store as a shared reference, so that it can be used both by `rs-matter` and the user
    let kv = stack.create_shared_kv(kv).unwrap();

    // == Step 5: ==
    // Run the Matter stack with our handler
    // Using `pin!` is completely optional, but reduces the size of the final future
    //
    // This step can be repeated in that the stack can be stopped and started multiple times, as needed.
    let mut matter = pin!(stack.run_coex(
        // The Matter stack needs to instantiate an `embassy-net` `Driver` and `Controller`
        EmbassyWifi::new(
            EspWifiDriver::new(peripherals.WIFI, peripherals.BT),
            weak_rand,
            true, // Use a random BLE address
            stack,
        ),
        // The crypto provider
        &crypto,
        // Our `AsyncHandler` + `AsyncMetadata` impl
        (NODE, handler),
        // The Matter stack needs a blob store to store its state
        &kv,
        // No user future to run; The LED task
        (),
    ));

    // == Step 6: ==
    // Setup the LED driver
    let receiver = channel.receiver();

    static RMT_BUFFER: StaticCell<[PulseCode; buffer_size_async(NUM_LEDS)]> = StaticCell::new();
    let rmt_buffer = RMT_BUFFER.init([PulseCode::default(); buffer_size_async(NUM_LEDS)]);

    let led_driver = rgb_led_driver::Driver::new(
        peripherals.RMT,
        peripherals.GPIO8.into(),
        receiver,
        rmt_buffer,
    );
    let mut led_task = pin!(led_driver.run());

    // == Step 7: ==
    // Setup reset button
    let mut button_reset = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Hold for 3 seconds to initiate a factory reset
    let mut reset_button_task = async || {
        loop {
            button_reset.wait_for_falling_edge().await;
            match select(button_reset.wait_for_rising_edge(), Timer::after_secs(3)).await {
                Either::First(_) => (),
                Either::Second(_) => {
                    info!("Factor reset has not been implemented");
                }
            }
        }
    };

    // == Step 7: ==
    // Run async tasks
    match select3(&mut matter, &mut led_task, &mut pin!(reset_button_task())).await {
        Either3::First(r) => {
            panic!("Matter thread exited! {:?}", r)
        }
        Either3::Second(_) => {
            panic!("LED thread exited!")
        }
        Either3::Third(_) => {
            panic!("Reset button thread exited!")
        }
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
    endpoints: &[
        EmbassyWifiMatterStack::<0, ()>::root_endpoint(),
        Endpoint::new(
            LIGHT_ENDPOINT_ID,
            devices!(DEV_TYPE_ENHANCED_COLOR_LIGHT),
            clusters!(
                desc::DescHandler::CLUSTER,
                OnOffHandler::<LedHandler<LedSender>, LedHandler<LedSender>>::CLUSTER,
                LevelControlHandler::<LedHandler<LedSender>, LedHandler<LedSender>>::CLUSTER
                color_control::ColorControlHandler::<LedHandler<LedSender>>::CLUSTER
            ),
        ),
    ],
};
