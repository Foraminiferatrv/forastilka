// use esp_hal::clock::CpuClock;
// use fugit::RateExtU32;
// use mipidsi::options::{RefreshOrder, Rotation, TearingEffect};
// pub struct Configuration {
//     pub cpu_freq: CpuClock,
//     pub spi_freq: fugit::HertzU32,
//     pub display_refresh_order: RefreshOrder,
//     pub display_rotation: Rotation,
//     pub display_tearing: TearingEffect,
// }
// /// Default configuration
// impl Default for Configuration {
//     fn default() -> Configuration {
//         Configuration {
//             spi_freq: 60.MHz(),
//             cpu_freq: CpuClock::_160MHz,
//             display_refresh_order: RefreshOrder::default(),
//             display_rotation: Rotation::Deg270,
//             display_tearing: TearingEffect::Off,
//         }
//     }
// }
// impl Configuration {
//     ///For creating configuration
//     pub fn new(
//         spi_freq: fugit::HertzU32,
//         cpu_freq: CpuClock,
//         display_refresh_order: RefreshOrder,
//         display_rotation: Rotation,
//         display_tearing: TearingEffect,
//     ) -> Configuration {
//         Configuration {
//             spi_freq,
//             cpu_freq,
//             display_refresh_order,
//             display_rotation,
//             display_tearing,
//         }
//     }
// }
