#![no_main]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![allow(warnings)]

use core::cell::RefCell;
use embassy_executor::Spawner;
use esp_backtrace as _; //Required as panic_handler

use embedded_sdmmc::Mode as SDMode;

use embassy_time::{Duration, Timer};
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_6X13_BOLD;
use embedded_graphics::mono_font::iso_8859_16::FONT_9X18_BOLD;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::Rectangle;
use embedded_hal::digital::OutputPin;
use embedded_sdmmc::VolumeIdx;

use esp_hal;
use esp_hal::gpio::Output;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;

use embedded_text;
use embedded_text::TextBox;
use embedded_text::alignment::{HorizontalAlignment, VerticalAlignment};
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
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
    let mut display = lilka.display;

    let mut volume0 = lilka.sd_volume_manager.open_volume(VolumeIdx(0)).unwrap();
    println!("Volume 0: {:?}", volume0);

    let syst = SystemTimer::new(lilka.peripherals.SYSTIMER);

    let timg0 = TimerGroup::new(lilka.peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let greeting_text = "Hello World...";

    let char_style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Justified)
        .vertical_alignment(VerticalAlignment::Bottom)
        .build();
    let bounds = Rectangle::new(Point::new(10, 20), Size::new(280, 0));
    let text_box = TextBox::with_textbox_style(greeting_text, bounds, char_style, textbox_style);

    text_box.draw(&mut display).unwrap();

    spawner.spawn(run()).ok();

    loop {
        println!("Ping!");

        Timer::after(Duration::from_millis(50_000)).await;
    }
}
