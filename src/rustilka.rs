use esp_hal::delay::Delay;
use esp_hal::gpio::{GpioPin, Input, Io, Level, Output};
use esp_hal::peripherals::{Peripherals, SPI2};
use mipidsi::{Builder, Display, NoResetPin};

use crate::cfg::Configuration;
use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};

use embedded_graphics::Drawable;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{DrawTarget, Point, Primitive};
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Triangle};

use embedded_hal::digital::OutputPin;
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_sdmmc::{Directory, SdCard, TimeSource, Timestamp, VolumeManager};

use esp_hal::DriverMode;
use esp_hal::clock::CpuClock;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::{BitOrder, Mode};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::Orientation;

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

// pub type LilkaDisplay = Display<
//     SPIInterface<
//         SpiDevice<
//             'static,
//             NoopRawMutex,
//             Spi<'static, dyn DriverMode>,
//             Output<'static>, // PIN 7
//             Output<'static>, // PIN 15
//             ST7789,
//             NoResetPin,
//         >,
//     >,
// >;
// pub type LilkaDisplay = Display<
//     SpiInterface<
//         SpiDevice<
//             'static,
//             NoopRawMutex,
//             Spi<'static, dyn DriverMode>,
//             Output<'static>, // PIN 7
//             Output<'static>, // PIN 15
//             ST7789,
//             NoResetPin,
//         >,
//     >,
// >;
pub struct Lilka {
    pub peripherals: Peripherals,
    pub delay: Delay,

    //Other fields
    pub state: InputState,
    // pub display,
    // pub sd_vol_mgr: SdVolMgr,
    // pub serial: Uart<'static, UART0, Async>,
    // pub battery: Battery,
    // pub buzzer: Buzzer,
    // pub i2s_tx: I2sTx<'static, I2S0, Channel0, Blocking>,
}

impl Lilka {
    pub fn new(lilka_config: Configuration) {
        //init log
        esp_println::logger::init_logger_from_env();

        // let peripherals = esp_hal::init(esp_hal::Config::default());
        // let peripherals = esp_hal::init(esp_hal::Config::default());
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz);
        let peripherals: Peripherals = esp_hal::init(config);
        // let peripherals = Peripherals::take();

        let io = Io::new(peripherals.IO_MUX);
        // let io = Io::new(peripherals.IO_MUX);

        //Display initialization

        // Disable the RTC and TIMG watchdog timers
        // println!("Disabling watchdog timers");
        let mut rtc = Rtc::new(peripherals.LPWR);
        let timer_group0 = TimerGroup::new(peripherals.TIMG0);
        let mut wdt0 = timer_group0.wdt;
        let timer_group1 = TimerGroup::new(peripherals.TIMG1);
        let mut wdt1 = timer_group1.wdt;
        rtc.swd.disable();
        rtc.rwdt.disable();
        wdt0.disable();
        wdt1.disable();

        // Define the delay struct, needed for the display driver
        let mut delay = Delay::new();

        //
        // Define the Data/Command select pin as a digital output
        let disp_dc = Output::new(peripherals.GPIO15, Level::Low);

        let mosi = peripherals.GPIO17;
        let miso = peripherals.GPIO8;
        let clk = peripherals.GPIO18;

        let mut spi_config: Config = Config::default();
        spi_config.frequency = lilka_config.spi_freq;
        spi_config.read_bit_order = BitOrder::MsbFirst;
        spi_config.write_bit_order = BitOrder::MsbFirst;
        spi_config.mode = Mode::_0;

        //DANGER peripherals.SPI2.into()
        let spi = Spi::new(peripherals.SPI2, spi_config)
            .unwrap()
            .with_sck(clk)
            .with_mosi(mosi)
            .with_miso(miso);

        let cs_output = Output::new(peripherals.GPIO7, Level::Low);
        let spi_device = ExclusiveDevice::new_no_delay(spi, cs_output).unwrap();

        let mut buffer = [0_u8; 512];

        // Define the display interface with no chip select
        let di = SpiInterface::new(spi_device, disp_dc, &mut buffer);

        // Define the display from the display interface and initialize it
        // println!("Display initialization");
        let mut display = Builder::new(ST7789, di)
            .display_size(240, 280)
            .orientation(Orientation::new().rotate(lilka_config.display_rotation))
            .init(&mut delay)
            .unwrap();

        // Make the display all black
        display.clear(Rgb565::BLACK).unwrap();

        // Draw a smiley face with white eyes and a red mouth
        draw_smiley(&mut display).unwrap();

        loop {
            // println!("Display initialization");
        }

        // let spi = NoopMutex::new(RefCell::new(spi));
        // let spi_bus = SPI_BUS.init(spi_bus);
        // let disp_spi = SpiDevice::new(spi_bus, disp_cs);
        // let di = SPIInterface::new(disp_spi, disp_dc);
        //
    }
}

fn draw_smiley<T: DrawTarget<Color = Rgb565>>(display: &mut T) -> Result<(), T::Error> {
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

    Ok(())
}
