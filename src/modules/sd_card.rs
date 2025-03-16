// use core::cell::RefCell;
// use embedded_hal_bus::spi::RefCellDevice;
// use embedded_sdmmc::{Mode as SDMode, SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};
// use esp_hal::Blocking;
// use esp_hal::delay::Delay;
// use esp_hal::gpio::{GpioPin, Level, Output, OutputConfig};
// use esp_hal::peripherals::Peripherals;
// use esp_hal::spi::master::Spi;
// use esp_println::println;
//
// //TODO: In progress (Split into separate files)
// type SDC<'a> = SdCard<RefCellDevice<'a, Spi<'a, Blocking>, Output<'a>, Delay>, Delay>;
//
// #[derive(Default)]
// pub struct DummyTimesource();
// impl TimeSource for DummyTimesource {
//     fn get_timestamp(&self) -> Timestamp {
//         Timestamp {
//             year_since_1970: 0,
//             zero_indexed_month: 0,
//             zero_indexed_day: 0,
//             hours: 0,
//             minutes: 0,
//             seconds: 0,
//         }
//     }
// }
//
// pub type SdVolMgr<'a> = VolumeManager<SDC<'a>, DummyTimesource>;
//
// // #[derive(Debug)]
// pub struct LilkaSDCard {
//     pub volume_manager: SdVolMgr<'static>,
// }
//
// impl LilkaSDCard {
//     pub fn init(
//         spi_bus: &'static mut RefCell<Spi<Blocking>>,
//         sd_cs_pin: GpioPin<16>,
//         delay: Delay,
//     ) -> Result<Self, &'static str> {
//         //===SD Card initialization===
//         // let sd_spi = SpiDevice::new(spi_bus, sd_cs);
//         let sd_cs = Output::new(sd_cs_pin, Level::Low, OutputConfig::default());
//         let sd_spi = match RefCellDevice::new(spi_bus, sd_cs, delay) {
//             Ok(spi_device) => {
//                 println!("\x1b[42m[OK]\x1b[0m SD SPI device init");
//                 spi_device
//             }
//             Err(e) => {
//                 println!("[ERROR] SD SPI init failed: {:?}", e);
//                 return Err("SD SPI init failed");
//             }
//         };
//
//         let sdcard = SdCard::new(sd_spi, delay);
//         println!("Card size is {} bytes", sdcard.num_bytes().unwrap());
//         let mut volume_manager = VolumeManager::new(sdcard, DummyTimesource::default());
//
//         Ok(Self { volume_manager })
//     }
// }
