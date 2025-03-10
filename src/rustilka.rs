use core::any::Any;
use core::cell::RefCell;
use core::fmt::Debug;
use embassy_sync::blocking_mutex::NoopMutex;
use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, GpioPin, Input, Io, Level, Output, OutputConfig, Pin, Pull};
use esp_hal::peripherals::{ADC1, Peripherals, SPI2};

use mipidsi::interface::{Interface, SpiInterface};
use mipidsi::models::{GC9107, ST7789};
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

use esp_hal::clock::{Clock, CpuClock};

use esp_hal::rtc_cntl::Rtc;
use esp_hal::spi::master::{Config, Spi};
use esp_hal::spi::{BitOrder, Mode};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{Async, Blocking, DriverMode};
use esp_println::{dbg, print, println};
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
        RefCellDevice<'static, Spi<'static, Blocking>, Output<'static>, NoDelay>,
        Output<'static>,
    >,
    ST7789,
    NoResetPin,
>;
// pub type LilkaDisplay = Display<
//     SpiInterface<
//         'static,
//         ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
//         Output<'static>,
//     >,
//     ST7789,
//     NoResetPin,
// >;

pub struct Buzzer(GpioPin<11>);
// pub struct Battery {
//     adc: Adc<'static, ADC1>,
//     adc_pin: AdcPin<GpioPin<3>, ADC1>,
// }
pub struct Lilka {
    pub peripherals: Peripherals,
    pub delay: Delay,
    pub display: LilkaDisplay,

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

impl Lilka {
    pub fn new(lilka_config: Configuration) -> Result<Self, &'static str> {
        println!("===Lilka init===");

        //init log
        esp_println::logger::init_logger_from_env();
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz);

        // let peripherals: Peripherals = esp_hal::init(config);
        let peripherals: Peripherals = esp_hal::init({
            let mut config = esp_hal::Config::default();
            config = config.with_cpu_clock(CpuClock::_160MHz);
            config
        });

        //Display initialization
        Output::new(
            peripherals.GPIO45,
            Level::High,
            OutputConfig::default().with_pull(Pull::Down),
        )
        .set_high();

        // Disable the RTC and TIMG watchdog timers
        println!("Disabling watchdog timers...");
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
        let disp_dc = Output::new(peripherals.GPIO15, Level::High, OutputConfig::default()); //good

        let disp_cs = Output::new(peripherals.GPIO7, Level::High, OutputConfig::default()); //good
        let mosi = peripherals.GPIO17; //good
        let miso = peripherals.GPIO8; //good
        let sck = peripherals.GPIO18; //good

        let spi_config: Config = Config::default()
            .with_frequency(Rate::from_mhz(60))
            .with_mode(Mode::_0);
        // .with_write_bit_order(BitOrder::MsbFirst)
        // .with_read_bit_order(BitOrder::MsbFirst);

        let spi = Spi::new(peripherals.SPI2, spi_config)
            .unwrap()
            .with_sck(sck)
            .with_mosi(mosi)
            .with_miso(miso);

        // let ref_spi = RefCell::new(spi);

        println!("CS pin: {:?}", disp_cs);

        // let spi_device = match ExclusiveDevice::new_no_delay(spi, disp_cs) {
        //     Ok(spi_device) => {
        //         println!("\x1b[42m[OK]\x1b[0m SPI device init");
        //         spi_device
        //     }
        //     Err(e) => {
        //         println!("[ERROR] Display init failed: {:?}", e);
        //         return Err("Display init failed");
        //     }
        // };

        let spi_static = SPI_CELL.init(RefCell::new(spi));
        let spi_device = match RefCellDevice::new_no_delay(spi_static, disp_cs) {
            Ok(spi_device) => {
                println!("\x1b[42m[OK]\x1b[0m SPI device init");
                spi_device
            }
            Err(e) => {
                println!("[ERROR] Display init failed: {:?}", e);
                return Err("Display init failed");
            }
        };
        // println!("\x1b[42m[SPI_DEVICE]\x1b[0m: {:?}", spi_device.inner());
        // log::info!("\x1b[42m[SPI_DEVICE]\x1b[0m: {:?}", spi_device);

        // log::info!("CS pin: {:?}", disp_cs);

        // let spi_bus = SPI_BUS.init(NoopMutex::new(RefCell::new(spi)));
        // let spi_device = SpiDevice::new(spi_bus, disp_cs);

        // Define the display interface with no chip select

        let display_buffer = DISPLAY_BUFFER_CELL.init([0_u8; 512]);

        let di = SpiInterface::new(spi_device, disp_dc, display_buffer);
        // println!("\x1b[42m[DISPLAY]\x1b[0m: {:?}", di.);

        // Define the display from the display interface and initialize it
        println!("Display initialization...");
        let mut display = match Builder::new(ST7789, di)
            .display_size(240, 280)
            .orientation(Orientation::new().rotate(lilka_config.display_rotation))
            .display_offset(0, 20)
            .invert_colors(ColorInversion::Inverted)
            .refresh_order(lilka_config.display_refresh_order)
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

        // println!("[display]: {:?}", display);

        // Make the display all white
        // display.set_tearing_effect(TearingEffect::Off).unwrap();
        // let dcs = unsafe { display.dcs() };
        // dcs.write_command(SoftReset).unwrap();
        // delay.delay_ms(150);

        display.clear(Rgb565::WHITE).unwrap();

        // println!("Display wake");
        // display.wake(&mut delay).unwrap();

        // Draw a smiley face with white eyes and a red mouth
        // println!("Draw smiley init");
        // log::info!("Draw smiley init");

        // draw_smiley(&mut display).unwrap();

        Ok(Lilka {
            peripherals: unsafe { Peripherals::steal() },
            delay,
            buzzer: Buzzer(peripherals.GPIO11),
            display,
        })

        // let spi = NoopMutex::new(RefCell::new(spi));
        // let spi_bus = SPI_BUS.init(spi_bus);
        // let disp_spi = SpiDevice::new(spi_bus, disp_cs);
        // let di = SPIInterface::new(disp_spi, disp_dc);
        //
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
