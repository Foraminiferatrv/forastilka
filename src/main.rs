#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

#![allow(warnings)]

use embassy_executor::Spawner;
// mod cfg;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use esp_backtrace as _;

use esp_hal;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Level, Output};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;

use esp_hal_embassy;
use esp_hal_embassy::Executor;
use esp_println::println;

use xtensa_lx_rt::entry;
mod cfg;
mod rustilka;

use cfg::Configuration;
use rustilka::Lilka;

#[embassy_executor::task]
async fn run() {
    loop {
        esp_println::println!("Hello world from task!");
        Timer::after(Duration::from_millis(1_000)).await;
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

    let timg0 = TimerGroup::new(lilka.peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    spawner.spawn(run()).ok();

    // let mut buzzer = Output::new(lilka.peripherals.GPIO11, Level::High);

    // spawner.spawn(buzz(buzzer)).ok();

    loop {
        println!("Ping!");

        // buzzer.set_high();
        // buzzer_delay.delay_millis(1000);
        // buzzer.set_low();

        Timer::after(Duration::from_millis(10_000)).await;
    }
}
