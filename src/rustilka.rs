use core::cell::RefCell;
use core::fmt::Debug;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex, RawMutex};
use embassy_sync::blocking_mutex::{Mutex, NoopMutex};
use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, GpioPin, Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::peripherals::{Peripherals, SPI1, SPI2};

use mipidsi::interface::{Interface, SpiInterface};
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, TearingEffect};
use mipidsi::{Builder, Display, NoResetPin};

use crate::cfg::Configuration;

use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{DrawTarget, Point, Primitive};
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Triangle};

use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;
use embedded_hal_bus::spi::{CriticalSectionDevice, ExclusiveDevice, RefCellDevice};
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeManager};
use esp_hal::clock::CpuClock;

// use embassy_sync::mutex::Mutex;
use esp_hal::spi::Mode;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::time::Rate;
use esp_hal::{Async, Blocking, DriverMode};
use esp_println::{dbg, println};
use static_cell::{ConstStaticCell, StaticCell};

pub struct InputState {
    pub up: Input<'static>,     // GPIO38
    pub down: Input<'static>,   // GPIO41
    pub left: Input<'static>,   // GPIO39
    pub right: Input<'static>,  // GPIO40
    pub a: Input<'static>,      // GPIO5
    pub b: Input<'static>,      // GPIO6
    pub c: Input<'static>,      // GPIO10
    pub d: Input<'static>,      // GPIO9
    pub select: Input<'static>, // GPIO0
    pub start: Input<'static>,  // GPIO4
}

pub type LilkaDisplay =
    Display<SpiInterface<'static, RefCellDevice<'static, Spi<'static, Async>, Output<'static>, Delay>, Output<'static>>, ST7789, NoResetPin>;
// pub type LilkaDisplayMutex = &'static mut EmbassyMutex<CriticalSectionRawMutex, &'static mut LilkaDisplay>;

pub struct Buzzer(Output<'static>);
// pub struct Battery {
//     adc: Adc<'static, ADC1>,
//     adc_pin: AdcPin<GpioPin<3>, ADC1>,
// }
pub struct Lilka {
    pub peripherals: Peripherals,
    pub delay: Delay,
    // pub display: &'static mut Mutex<LilkaDisplay>,
    pub display: LilkaDisplay,

    //Other fields
    pub input_state: InputState,
    pub sd_volume_manager: SdVolMgr,
    // pub serial: Uart<'static, UART0, Async>,
    // pub battery: Battery,
    pub buzzer: Buzzer,
    // pub i2s_tx: I2sTx<'static, I2S0, Channel0, Blocking>,
}
pub static EXECUTOR: StaticCell<esp_hal_embassy::Executor> = StaticCell::new();

static DISPLAY_BUFFER_CELL: StaticCell<[u8; 512]> = StaticCell::new();

type Spi2Bus = Mutex<NoopRawMutex, RefCell<Spi<'static, Async>>>;
//==SD Card==
pub struct SD;

#[derive(Default)]
pub struct DummyTimesource();

impl TimeSource for DummyTimesource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
type SDC<'a> = SdCard<RefCellDevice<'a, Spi<'a, Async>, Output<'a>, Delay>, Delay>;

pub type SdVolMgr = VolumeManager<SDC<'static>, DummyTimesource>;

impl Lilka {
    pub async fn new(lilka_config: Configuration) -> Result<Self, &'static str> {
        println!("===Lilka init===");

        // let peripherals: Peripherals = esp_hal::init(config);
        let peripherals: Peripherals = esp_hal::init({
            let mut config = esp_hal::Config::default();
            config = config.with_cpu_clock(CpuClock::_160MHz);
            config
        });

        //Display initialization
        let mut pin46 = Output::new(
            peripherals.GPIO46,
            Level::High,
            OutputConfig::default().with_drive_mode(DriveMode::PushPull).with_pull(Pull::Down),
        );
        pin46.set_high();

        dbg!(pin46.output_level());
        // Define the delay struct, needed for the display driver
        let mut delay = Delay::new();

        //
        // Define the Data/Command select pin as a digital output
        let disp_dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default().with_drive_mode(DriveMode::PushPull));

        let disp_cs = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default().with_drive_mode(DriveMode::PushPull));

        let mosi = peripherals.GPIO17;
        let miso = peripherals.GPIO8;
        let sck = peripherals.GPIO18;

        let spi_config: Config = Config::default().with_frequency(Rate::from_mhz(60)).with_mode(Mode::_0);

        let spi = Spi::new(peripherals.SPI2, spi_config)
            .unwrap()
            .with_sck(sck)
            .with_mosi(mosi)
            .with_miso(miso)
            .into_async();

        static SPI_BUS: StaticCell<Spi2Bus> = StaticCell::new();
        let mut spi_bus = SPI_BUS.init(Mutex::new(RefCell::new(spi)));

        let spi_device = match RefCellDevice::new(spi_bus.borrow(), disp_cs, delay) {
            Ok(spi_device) => {
                println!("\x1b[42m[OK]\x1b[0m SPI device init");
                spi_device
            }
            Err(e) => {
                println!("[ERROR] SPI device init failed: {:?}", e);
                return Err("SPI device init failed");
            }
        };

        // Define the display interface with no chip select
        let display_buffer = DISPLAY_BUFFER_CELL.init([0_u8; 512]);
        let di = SpiInterface::new(spi_device, disp_dc, display_buffer);

        // Define the display from the display interface and initialize it
        println!("Display initialization...");

        let mut display = match Builder::new(ST7789, di)
            .refresh_order(lilka_config.display_refresh_order)
            .invert_colors(ColorInversion::Inverted)
            .display_size(240, 280)
            .orientation(Orientation::new().rotate(lilka_config.display_rotation))
            .display_offset(0, 20)
            .color_order(mipidsi::options::ColorOrder::Rgb)
            .init(&mut delay)
        {
            Ok(display) => {
                // println!("[OK] Display init");
                println!("\x1b[42m[OK]\x1b[0m Display init");
                display
            }
            Err(e) => {
                println!("[ERROR] Display init failed: {:?}", e);
                return Err("Display init failed");
            }
        };

        display.set_tearing_effect(TearingEffect::Off).unwrap();

        display.clear(Rgb565::BLACK).unwrap();

        //===Buzzer initialization===f
        let mut buzzer = Output::new(peripherals.GPIO11, Level::Low, OutputConfig::default().with_drive_mode(DriveMode::PushPull));

        //===SD Card initialization===
        let sd_cs = Output::new(peripherals.GPIO16, Level::Low, OutputConfig::default());
        let sd_spi = match RefCellDevice::new(spi_bus.borrow(), sd_cs, delay) {
            Ok(spi_device) => {
                println!("\x1b[42m[OK]\x1b[0m SD SPI device init");
                spi_device
            }
            Err(e) => {
                println!("[ERROR] SD SPI init failed: {:?}", e);
                return Err("SD SPI init failed");
            }
        };

        let sdcard = SdCard::new(sd_spi, delay);
        println!("Card size is {} bytes", sdcard.num_bytes().unwrap());
        let mut sd_volume_manager = VolumeManager::new(sdcard, DummyTimesource::default());

        //===Button Inputs initialization===
        let input_state = InputState {
            a: Input::new(peripherals.GPIO5, InputConfig::default().with_pull(Pull::Up)),
            b: Input::new(peripherals.GPIO6, InputConfig::default().with_pull(Pull::Up)),
            c: Input::new(peripherals.GPIO10, InputConfig::default().with_pull(Pull::Up)),
            d: Input::new(peripherals.GPIO9, InputConfig::default().with_pull(Pull::Up)),

            up: Input::new(peripherals.GPIO38, InputConfig::default().with_pull(Pull::Up)),
            down: Input::new(peripherals.GPIO41, InputConfig::default().with_pull(Pull::Up)),
            left: Input::new(peripherals.GPIO39, InputConfig::default().with_pull(Pull::Up)),
            right: Input::new(peripherals.GPIO40, InputConfig::default().with_pull(Pull::Up)),

            select: Input::new(peripherals.GPIO0, InputConfig::default().with_pull(Pull::Up)),
            start: Input::new(peripherals.GPIO4, InputConfig::default().with_pull(Pull::Up)),
        };

        Ok(Lilka {
            peripherals: unsafe { Peripherals::steal() },
            delay,
            buzzer: Buzzer(buzzer),
            input_state,
            sd_volume_manager,
            display,
        })
    }
}
