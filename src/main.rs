#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![allow(warnings)]
use core::cell::RefCell;
use embassy_executor::Spawner;
use embedded_sdmmc::Mode as SDMode;
// mod cfg;
use embassy_time::{Duration, Timer};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_hal::digital::OutputPin;
use embedded_sdmmc::VolumeIdx;
use esp_backtrace as _;

use esp_hal;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Level, Output};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;

use esp_hal_embassy;
use esp_hal_embassy::Executor;
use esp_println::{print, println};

use xtensa_lx_rt::entry;
mod cfg;
mod modules;
mod rustilka;

use cfg::Configuration;
use rustilka::Lilka;

#[embassy_executor::task]
async fn run() {
    loop {
        esp_println::println!("Hello world from task!");
        Timer::after(Duration::from_millis(40_000)).await;
    }
}
#[embassy_executor::task]
async fn buzz(mut buzzer: Output<'static>) {
    loop {
        esp_println::println!("Buzzin!");
        buzzer.set_high();
        Timer::after(Duration::from_millis(500)).await;
        buzzer.set_high();
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    let mut lilka = Lilka::new(Configuration::default()).unwrap();

    //SD CARD TESTS:
    let mut volume0 = lilka.sd_volume_manager.open_volume(VolumeIdx(0)).unwrap();
    println!("Volume 0: {:?}", volume0);
    let mut root_dir = RefCell::new(volume0.open_root_dir().unwrap());
    const FILE_TO_CREATE: &str = "CREATE3.TXT";

    {
        let mut f = root_dir
            .get_mut()
            .open_file_in_dir(FILE_TO_CREATE, SDMode::ReadWriteCreate)
            .unwrap();
        match f.write(b"Hello, this is a new file 3 on disk\r\n") {
            Ok(_) => println!("File written"),
            Err(e) => println!("Error writing file: {:?}", e),
        };
    }

    {
        let mut my_file = root_dir
            .get_mut()
            .open_file_in_dir(FILE_TO_CREATE, SDMode::ReadOnly)
            .unwrap();

        while !my_file.is_eof() {
            let mut buffer = [0u8; 32];
            let num_read = my_file.read(&mut buffer).unwrap();
            for b in &buffer[0..num_read] {
                print!("{}", *b as char);
            }
        }
    }

    let syst = SystemTimer::new(lilka.peripherals.SYSTIMER);

    let timg0 = TimerGroup::new(lilka.peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    spawner.spawn(run()).ok();

    loop {
        println!("Ping!");

        Timer::after(Duration::from_millis(50_000)).await;
    }
}
