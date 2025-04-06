#![no_main]
#![feature(async_closure)]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![allow(warnings)]

use core::cell::{RefCell, RefMut};
use core::ops::DerefMut;
use embassy_executor::Spawner;
use embassy_futures::join::{join, join_array};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
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
use esp_hal::gpio::{Input, Output};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;

use embedded_text;
use embedded_text::TextBox;
use embedded_text::alignment::{HorizontalAlignment, VerticalAlignment};
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use esp_hal_embassy;
use esp_hal_embassy::Executor;
use esp_println::{print, println};
use static_cell::{ConstStaticCell, StaticCell, make_static};
use xtensa_lx_rt::entry;
mod cfg;
mod modules;
mod rustilka;

use crate::rustilka::{EXECUTOR, InputState, LilkaDisplay};
use cfg::Configuration;
use rustilka::Lilka;

pub static DISPLAY: ConstStaticCell<Mutex<CriticalSectionRawMutex, RefCell<Option<&mut LilkaDisplay>>>> = ConstStaticCell::new(Mutex::new(RefCell::new(None)));

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

fn reset_screen_text() {
    let display = DISPLAY.try_take().unwrap();
    //FOR TESTING
    display.lock(|mut d| {
        let mut disp: RefMut<Option<&mut LilkaDisplay>> = d.borrow_mut();

        disp.as_deref_mut().unwrap().clear(Rgb565::BLACK).ok();

        let char_style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(HorizontalAlignment::Justified)
            .vertical_alignment(VerticalAlignment::Bottom)
            .build();
        let bounds = Rectangle::new(Point::new(10, 20), Size::new(280, 0));
        let text_box = TextBox::with_textbox_style("Waiting for input...", bounds, char_style, textbox_style);

        text_box.draw(disp.as_deref_mut().unwrap()).unwrap();
    });
}
// fn draw_text(text: &str, display: &mut LilkaDisplay) {
//     let char_style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);
//     let textbox_style = TextBoxStyleBuilder::new()
//         .height_mode(HeightMode::FitToText)
//         .alignment(HorizontalAlignment::Justified)
//         .vertical_alignment(VerticalAlignment::Bottom)
//         .build();
//     let bounds = Rectangle::new(Point::new(10, 20), Size::new(280, 0));
//     let text_box = TextBox::with_textbox_style(text, bounds, char_style, textbox_style);
//
//     display.clear(Rgb565::BLACK).ok();
//
//     text_box.draw(display).unwrap();
// }

async fn draw_text_with_timeout(text: &str, display: &mut LilkaDisplay) {
    draw_text(text, display);
    Timer::after(Duration::from_millis(1000)).await;
    reset_screen_text();
}
// #[embassy_executor::task]
// async fn test_input(mut input_state: InputState, display: &'static mut LilkaDisplay) {
//     println!("test_input");
//
//     join_array([
//         button(input_state.a, "A", display),
//         button(input_state.b, "B", display),
//         button(input_state.c, "C", display),
//         button(input_state.d, "D", display),
//         button(input_state.select, "Select", display),
//         button(input_state.start, "Start", display),
//         button(input_state.up, "Up", display),
//         button(input_state.down, "Down", display),
//         button(input_state.left, "Left", display),
//         button(input_state.right, "Right", display),
//     ]).await;
//
//     println!("END TASK");
// }

async fn button(mut input: Input<'_>, name: &str, display: &mut LilkaDisplay) {
    loop {
        input.wait_for_low().await;
        println!("Button pressed: {}", name);
        draw_text(name, display).await;
        Timer::after_millis(200).await;
        input.wait_for_high().await;
    }
}

async fn draw_text(text: &str, display: &mut LilkaDisplay) {
    let char_style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Justified)
        .vertical_alignment(VerticalAlignment::Bottom)
        .build();
    let bounds = Rectangle::new(Point::new(10, 20), Size::new(280, 0));
    let text_box = TextBox::with_textbox_style(text, bounds, char_style, textbox_style);

    display.clear(Rgb565::BLACK).ok();

    text_box.draw(display).unwrap();
}
#[embassy_executor::task]
async fn render(display: &'static mut LilkaDisplay) {
    loop {
        draw_text("Hello World!", display).await;
        Timer::after_millis(1000).await;
        draw_text("Hello World 2!", display).await;
        Timer::after_millis(1000).await;
    }
}

static DISPLAY_CELL: StaticCell<LilkaDisplay> = StaticCell::new();
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let mut lilka = Lilka::new(Configuration::default()).await.unwrap();
    let display = DISPLAY_CELL.init(lilka.display);

    let input_state = lilka.input_state;

    let mut volume0 = lilka.sd_volume_manager.open_volume(VolumeIdx(0)).unwrap();

    let syst = SystemTimer::new(lilka.peripherals.SYSTIMER);

    let timg0 = TimerGroup::new(lilka.peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // spawner.spawn(test_input(input_state, display)).unwrap();
    spawner.spawn(render(display)).unwrap();
}
