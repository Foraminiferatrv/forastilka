// use core::cell::RefCell;
// use core::fmt::Debug;
//
// use esp_hal::delay::Delay;
// use esp_hal::gpio::{DriveMode, Input, Level, Output, OutputConfig, Pull};
// use esp_hal::peripherals::Peripherals;
//
// use mipidsi::interface::{Interface, SpiInterface};
// use mipidsi::models::ST7789;
// use mipidsi::options::{ColorInversion, Orientation, TearingEffect};
// use mipidsi::{Builder, Display, NoResetPin};
//
// use crate::cfg::Configuration;
//
// use embedded_graphics::Drawable;
// use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
// use embedded_graphics::prelude::{DrawTarget, Point, Primitive};
// use embedded_graphics::primitives::{Circle, PrimitiveStyle, Triangle};
//
// use embedded_hal::digital::OutputPin;
// use embedded_hal_bus::spi::RefCellDevice;
// use esp_hal::clock::CpuClock;
//
// use crate::modules::sd_card::{LilkaSDCard, SdVolMgr};
// use esp_hal::Blocking;
// use esp_hal::spi::Mode;
// use esp_hal::spi::master::{Config, Spi};
// use esp_hal::time::Rate;
// use esp_println::{dbg, println};
// use static_cell::StaticCell;
//

//TODO: In progress
// type LilkaDisplay<'a> = Display<
//     SpiInterface<'a, RefCellDevice<'a, Spi<'a, Blocking>, &'a mut Output<'a>, Delay>, Output<'a>>,
//     ST7789,
//     NoResetPin,
// >;
//
// // pub trait LilkaDisplay {
// //     fn init(
// //         peripherals: Peripherals,
// //         spi_bus: &mut RefCell<Spi<Blocking>>,
// //         lilka_config: Configuration,
// //         delay: Delay,
// //     ) -> Result<Self, &'static str>
// //     where
// //         Self: Sized;
// // }
//
// static SPI_CELL: StaticCell<RefCell<Spi<'static, Blocking>>> = StaticCell::new();
// static DISPLAY_BUFFER_CELL: StaticCell<[u8; 512]> = StaticCell::new();
//
// pub fn init_display(
//     peripherals: Peripherals,
//     spi_bus: &mut RefCell<Spi<Blocking>>,
//     lilka_config: Configuration,
//     mut delay: &mut Delay,
// ) -> Result<
//     Display<
//         SpiInterface<
//             'static,
//             RefCellDevice<'static, Spi<'static, Blocking>, Output<'static>, &'static mut Delay>,
//             Output<'static>,
//         >,
//         ST7789,
//         NoResetPin,
//     >,
//     &'static str,
// > {
//     // let mut delay = Delay::new();
//
//     //Display initialization
//     let mut pin46 = Output::new(
//         peripherals.GPIO46,
//         Level::High,
//         OutputConfig::default()
//             .with_drive_mode(DriveMode::PushPull)
//             .with_pull(Pull::Down),
//     );
//     pin46.set_high();
//
//     // Disable the RTC and TIMG watchdog timers
//     // println!("Disabling watchdog timers...");
//     // let mut rtc = Rtc::new(peripherals.LPWR);
//     // let timer_group0 = TimerGroup::new(peripherals.TIMG0);
//     // let mut wdt0 = timer_group0.wdt;
//     // let timer_group1 = TimerGroup::new(peripherals.TIMG1);
//     // let mut wdt1 = timer_group1.wdt;
//     // rtc.swd.disable();
//     // rtc.rwdt.disable();
//     // wdt0.disable();
//     // wdt1.disable();
//
//     // Define the Data/Command select pin as a digital output
//     let mut disp_dc: Output<'static> = Output::new(
//         peripherals.GPIO15,
//         Level::Low,
//         OutputConfig::default().with_drive_mode(DriveMode::PushPull),
//     );
//
//     let mut disp_cs: Output<'static> = Output::new(
//         peripherals.GPIO7,
//         Level::Low,
//         OutputConfig::default().with_drive_mode(DriveMode::PushPull),
//     );
//
//     // let spi_device = match RefCellDevice::new_no_delay(spi_bus, disp_cs) {
//     let spi_device = match RefCellDevice::new(spi_bus, disp_cs, delay) {
//         Ok(spi_device) => {
//             println!("\x1b[42m[OK]\x1b[0m SPI device init");
//             spi_device
//         }
//         Err(e) => {
//             println!("[ERROR] Display init failed: {:?}", e);
//             return Err("Display init failed");
//         }
//     };
//
//     // Define the display interface with no chip select
//     let display_buffer = DISPLAY_BUFFER_CELL.init([0_u8; 512]);
//
//     let di = SpiInterface::new(spi_device, disp_dc, display_buffer);
//
//     // Define the display from the display interface and initialize it
//     println!("Display initialization...");
//
//     let mut display = match Builder::new(ST7789, di)
//         .refresh_order(lilka_config.display_refresh_order)
//         .invert_colors(ColorInversion::Inverted)
//         .display_size(240, 280)
//         .orientation(Orientation::new().rotate(lilka_config.display_rotation))
//         .display_offset(0, 20)
//         .color_order(mipidsi::options::ColorOrder::Rgb)
//         .init(&mut delay)
//     {
//         Ok(display) => {
//             // println!("[OK] Display init");
//             println!("\x1b[42m[OK]\x1b[0m Display init");
//             display
//         }
//         Err(e) => {
//             println!("[ERROR] Display init failed: {:?}", e);
//             return Err("Display init failed");
//         }
//     };
//
//     Ok(display)
// }
