#![no_std]
#![no_main]
// mod cfg;
use esp_backtrace as _;

use esp_println::println;
use xtensa_lx_rt::entry;

mod rustilka;
#[entry]
fn main() -> ! {
    // println!("Hello, world!");

    loop {}
}
