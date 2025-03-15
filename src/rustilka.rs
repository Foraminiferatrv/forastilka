use core::cell::RefCell;
use core::fmt::Debug;

use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, GpioPin, Input, Level, Output, OutputConfig, Pin, Pull};
use esp_hal::peripherals::Peripherals;

use mipidsi::interface::{Interface, SpiInterface};
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, TearingEffect};
use mipidsi::{Builder, Display, NoResetPin};

use crate::cfg::Configuration;

use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{DrawTarget, Point, Primitive, Size};
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Rectangle, Triangle};

use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay, RefCellDevice};
use embedded_sdmmc::{Mode as SDMode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
use esp_hal::clock::{Clock, CpuClock};

use esp_hal::rtc_cntl::Rtc;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::{BitOrder, Mode};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{Async, Blocking, DriverMode};
use esp_println::{dbg, print, println};
use log::debug;
use mipidsi::dcs::{InterfaceExt, SoftReset};
use static_cell::StaticCell;

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

pub type LilkaDisplay = Display<
    SpiInterface<
        'static,
        RefCellDevice<'static, Spi<'static, Blocking>, Output<'static>, Delay>,
        Output<'static>,
    >,
    ST7789,
    NoResetPin,
>;
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

type SDC = SdCard<
    SpiInterface<
        'static,
        RefCellDevice<'static, Spi<'static, Blocking>, Output<'static>, Delay>,
        Output<'static>,
    >,
    Delay,
>;
pub type SdVolMgr = VolumeManager<SDC, DummyTimesource>;

pub struct Buzzer(Output<'static>);
// pub struct Battery {
//     adc: Adc<'static, ADC1>,
//     adc_pin: AdcPin<GpioPin<3>, ADC1>,
// }
pub struct Lilka {
    pub peripherals: Peripherals,
    pub delay: Delay,
    // pub display: LilkaDisplay,

    //Other fields
    // pub state: InputState,
    // pub display,
    // pub sd_vol_mgr: SdVolMgr,
    // pub serial: Uart<'static, UART0, Async>,
    // pub battery: Battery,
    pub buzzer: Buzzer,
    // pub i2s_tx: I2sTx<'static, I2S0, Channel0, Blocking>,
}
pub static EXECUTOR: StaticCell<esp_hal_embassy::Executor> = StaticCell::new();
static DISPLAY_BUFFER_CELL: StaticCell<[u8; 512]> = StaticCell::new();
static SPI_CELL: StaticCell<RefCell<Spi<'static, Blocking>>> = StaticCell::new();
pub struct SD;
impl Lilka {
    pub fn new(lilka_config: Configuration) -> Result<Self, &'static str> {
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
            OutputConfig::default()
                .with_drive_mode(DriveMode::PushPull)
                .with_pull(Pull::Down),
        );
        pin46.set_high();

        dbg!(pin46.output_level());

        // Disable the RTC and TIMG watchdog timers
        // println!("Disabling watchdog timers...");
        // let mut rtc = Rtc::new(peripherals.LPWR);
        // let timer_group0 = TimerGroup::new(peripherals.TIMG0);
        // let mut wdt0 = timer_group0.wdt;
        // let timer_group1 = TimerGroup::new(peripherals.TIMG1);
        // let mut wdt1 = timer_group1.wdt;
        // rtc.swd.disable();
        // rtc.rwdt.disable();
        // wdt0.disable();
        // wdt1.disable();

        // Define the delay struct, needed for the display driver
        let mut delay = Delay::new();

        //
        // Define the Data/Command select pin as a digital output
        let mut disp_dc = Output::new(
            peripherals.GPIO15,
            Level::Low,
            OutputConfig::default().with_drive_mode(DriveMode::PushPull),
        );

        let mut disp_cs = RefCell::new(Output::new(
            peripherals.GPIO7,
            Level::Low,
            OutputConfig::default().with_drive_mode(DriveMode::PushPull),
        )); //good pin

        dbg!(disp_dc.output_level());

        let mosi = peripherals.GPIO17; //good
        let miso = peripherals.GPIO8; //good
        let sck = peripherals.GPIO18; //good

        let spi_config: Config = Config::default()
            .with_frequency(Rate::from_mhz(60))
            .with_mode(Mode::_0);

        let spi = Spi::new(peripherals.SPI2, spi_config)
            .unwrap()
            .with_sck(sck)
            .with_mosi(mosi)
            .with_miso(miso);

        let spi_bus = SPI_CELL.init(RefCell::new(spi));
        // let spi_device = match RefCellDevice::new_no_delay(spi_bus, disp_cs) {
        let spi_device = match RefCellDevice::new(spi_bus, disp_cs.get_mut(), delay) {
            Ok(spi_device) => {
                println!("\x1b[42m[OK]\x1b[0m SPI device init");
                spi_device
            }
            Err(e) => {
                println!("[ERROR] Display init failed: {:?}", e);
                return Err("Display init failed");
            }
        };

        // Define the display interface with no chip select
        let display_buffer = DISPLAY_BUFFER_CELL.init([0_u8; 512]);

        let di = SpiInterface::new(spi_device, disp_dc, display_buffer);

        // Define the display from the display interface and initialize it
        println!("Display initialization...");

        let mut display = match Builder::new(ST7789, di)
            // .reset_pin(rst)
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

        dbg!(display.is_sleeping());

        display.set_tearing_effect(TearingEffect::Off).unwrap();

        // Make the display all black
        display.clear(Rgb565::BLACK).unwrap();
        // Draw a smiley face with white eyes and a red mouth

        draw_smiley(&mut display).unwrap();

        //===Buzzer initialization===
        let mut buzzer = Output::new(
            peripherals.GPIO11,
            Level::Low,
            OutputConfig::default().with_drive_mode(DriveMode::PushPull),
        );

        //===SD Card initialization===
        let sd_cs = Output::new(peripherals.GPIO16, Level::Low, OutputConfig::default());
        // let sd_spi = SpiDevice::new(spi_bus, sd_cs);
        let sd_spi = match RefCellDevice::new(spi_bus, sd_cs, delay) {
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
        let mut volume_mgr = VolumeManager::new(sdcard, DummyTimesource::default());
        let mut volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
        println!("Volume 0: {:?}", volume0);

        let mut root_dir = RefCell::new(volume0.open_root_dir().unwrap());

        //SD CARD TESTS:
        const FILE_TO_CREATE: &str = "CREATE.TXT";

        {
            let mut f = root_dir
                .get_mut()
                .open_file_in_dir(FILE_TO_CREATE, SDMode::ReadWriteCreate)
                .unwrap();
            match f.write(b"Hello, this is a new file on disk\r\n") {
                Ok(_) => println!("File written"),
                Err(e) => println!("Error writing file: {:?}", e),
            };
        }

        {
            let mut my_file = match root_dir
                .get_mut()
                .open_file_in_dir(FILE_TO_CREATE, SDMode::ReadOnly)
            {
                Ok(file) => file,
                Err(e) => {
                    println!("Error opening file: {:?}", e);
                    return Err("Error opening file");
                }
            };

            while !my_file.is_eof() {
                let mut buffer = [0u8; 32];
                let num_read = my_file.read(&mut buffer).unwrap();
                for b in &buffer[0..num_read] {
                    print!("{}", *b as char);
                }
            }
        }

        //SD CARD TESTS END:

        Ok(Lilka {
            peripherals: unsafe { Peripherals::steal() },
            delay,
            buzzer: Buzzer(buzzer),
            // display,
        })
    }
}

fn draw_smiley<T: DrawTarget<Color = Rgb565>>(display: &mut T) -> Result<(), T::Error> {
    println!("Draw start");

    // Draw the left eye as a circle located at (50, 100), with a diameter of 40, filled with white
    Circle::new(Point::new(50, 100), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw the right eye as a circle located at (50, 200), with a diameter of 40, filled with white
    Circle::new(Point::new(50, 200), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw an upside down red triangle to represent a smiling mouth
    Triangle::new(
        Point::new(130, 140),
        Point::new(130, 200),
        Point::new(160, 170),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(display)?;

    // Cover the top part of the mouth with a black triangle so it looks closed instead of open
    Triangle::new(
        Point::new(130, 150),
        Point::new(130, 190),
        Point::new(150, 170),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(display)?;
    println!("Draw end");

    Ok(())
}
