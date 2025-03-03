use esp_hal::delay::Delay;
use esp_hal::gpio::{GpioPin, Input, Io, Level, Output};
use esp_hal::peripherals::{Peripherals, SPI2};
use mipidsi::{Builder, Display, NoResetPin};

use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_sync::blocking_mutex::{NoopMutex, raw::NoopRawMutex};
use embedded_hal::digital::OutputPin;
use embedded_sdmmc::{Directory, SdCard, TimeSource, Timestamp, VolumeManager};
use esp_hal::DriverMode;
use esp_hal::clock::CpuClock;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::spi::master::Spi;
use esp_hal::timer::timg::TimerGroup;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;

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
    pub fn new() {
        // let peripherals = esp_hal::init(esp_hal::Config::default());
        // let peripherals = esp_hal::init(esp_hal::Config::default());
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz);
        let peripherals: Peripherals = esp_hal::init(config);
        // let peripherals = Peripherals::take();

        // let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
        // let io = Io::new(peripherals.IO_MUX);

        //Display initialization

        // Disable the RTC and TIMG watchdog timers
        let mut rtc = Rtc::new(peripherals.LPWR);
        let timer_group0 = TimerGroup::new(peripherals.TIMG0);
        let mut wdt0 = timer_group0.wdt;
        let timer_group1 = TimerGroup::new(peripherals.TIMG1);
        let mut wdt1 = timer_group1.wdt;
        rtc.swd.disable();
        rtc.rwdt.disable();
        wdt0.disable();
        wdt1.disable();
        //
        // // Define the delay struct, needed for the display driver
        // let mut delay = Delay::new();
        //
        // // Define the Data/Command select pin as a digital output
        // let dc = Output::new(io.pins, Level::Low);
        //
        // let spi_bus = NoopMutex::new(RefCell::new(spi));
        // let spi_bus = SPI_BUS.init(spi_bus);
        // let disp_spi = SpiDevice::new(spi_bus, disp_cs);
        // let di = SPIInterface::new(disp_spi, disp_dc);
        //
        // let display = Builder::new(ST7789, di);
    }
}
