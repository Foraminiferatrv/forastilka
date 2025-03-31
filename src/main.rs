#![no_main]
#![feature(async_closure)]
#![no_std]
#![feature(impl_trait_in_assoc_type)]
#![allow(warnings)]

use core::cell::{RefCell, RefMut};
use core::ops::DerefMut;
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

use crate::rustilka::{DISPLAY, EXECUTOR, InputState, LilkaDisplay, LilkaDisplayMutex};
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
fn draw_text(text: &str) {
    //FOR TESTING
    let display = DISPLAY.try_take().unwrap();

    display.lock(|mut d| {
        let mut disp: RefMut<Option<&mut LilkaDisplay>> = d.borrow_mut();

        let char_style = MonoTextStyle::new(&FONT_9X18_BOLD, Rgb565::WHITE);
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(HorizontalAlignment::Justified)
            .vertical_alignment(VerticalAlignment::Bottom)
            .build();
        let bounds = Rectangle::new(Point::new(10, 20), Size::new(280, 0));
        let text_box = TextBox::with_textbox_style(text, bounds, char_style, textbox_style);

        disp.as_deref_mut().unwrap().clear(Rgb565::BLACK).ok();

        text_box.draw(disp.as_deref_mut().unwrap()).unwrap();
    });
}

async fn draw_text_with_timeout(text: &str) {
    draw_text(text);
    Timer::after(Duration::from_millis(1000)).await;
    reset_screen_text();
}
#[embassy_executor::task]
async fn test_input(input_state: InputState) {
    //INPUTS TEST
    let greeting_text = "Hello World...";

    if input_state.a.is_low() {
        println!("Button A pressed");
        println!("{:?}", input_state.a.level());
        draw_text_with_timeout("Button A pressed").await;
    }
    if input_state.b.is_low() {
        println!("Button B pressed");
        draw_text_with_timeout("Button B pressed").await;
    }
    if input_state.c.is_low() {
        println!("Button C pressed");
        draw_text_with_timeout("Button C pressed").await;
    }
    if input_state.d.is_low() {
        println!("Button D pressed");
        draw_text_with_timeout("Button D pressed").await;
    }
    if input_state.select.is_low() {
        println!("Button Select pressed");
        draw_text_with_timeout("Button Select pressed").await;
    }
    if input_state.start.is_low() {
        println!("Button Start pressed");
        draw_text_with_timeout("Button Start pressed").await;
    }

    if input_state.up.is_low() {
        println!("Button Up pressed");
        draw_text_with_timeout("Button Up pressed").await;
    }
    if input_state.down.is_low() {
        println!("Button Down pressed");
        draw_text_with_timeout("Button Down pressed").await;
    }
    if input_state.left.is_low() {
        println!("Button Left pressed");
        draw_text_with_timeout("Button Left pressed").await;
    }
    if input_state.right.is_low() {
        println!("Button Right pressed");
        draw_text_with_timeout("Button Right pressed").await;
    }
}

async fn button_pressed(input_state: InputState, display: &mut LilkaDisplay) {}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    let mut lilka = Lilka::new(Configuration::default()).await.unwrap();
    let input_state = lilka.input_state;

    let mut volume0 = lilka.sd_volume_manager.open_volume(VolumeIdx(0)).unwrap();

    let syst = SystemTimer::new(lilka.peripherals.SYSTIMER);

    let timg0 = TimerGroup::new(lilka.peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    let greeting_text = "Hello World...";

    draw_text("Hello from refactored");
    // reset_screen_text(display);

    // spawner.spawn(run()).ok();

    // 1. Create executor(s)
    // 2. spawn tasks
    // 3. run executor

    let executor = EXECUTOR.init(Executor::new());

    // spawner.spawn(test_input(&input_state, display)).unwrap();

    'start: loop {
        // Timer::after_millis(10u64).await;
    }
}
