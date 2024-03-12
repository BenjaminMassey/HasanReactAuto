use enigo::*;
use screenshots::Screen;
use std::{thread, time};
use raster::filter;
//use rust_ocr::png_to_text;

const CAPTURE_PATH: &str = "./capture.png";
const BROWSER_EXE: &str = "chrome.exe";
const HASAN_TWITCH_URL: &str = "https://www.twitch.tv/hasanabi";
const CURRENT_URL_TOP_LEFT: (usize, usize) = (78, 166);
const CURRENT_URL_BOTTOM_RIGHT: (usize, usize) = (589, 185);
const CAPTIONS_START_LOC: (i32, i32) = (777, 700);
const CAPTIONS_END_LOC: (i32, i32) = (777, 0);
const CAPTIONS_TOP_LEFT: (usize, usize) = (426, 32);
const CAPTIONS_BOTTOM_RIGHT: (usize, usize) = (934, 82);

fn main() {
    let mut enigo = Enigo::new();
    start_stream(&mut enigo);
    let screen = Screen::from_point(0, 0).unwrap();
    screenshot_current_url(screen, &CAPTURE_PATH);
    let mut image = raster::open(&CAPTURE_PATH).unwrap();
    filter::sharpen(&mut image).unwrap();
    raster::save(&image, &CAPTURE_PATH).unwrap();
    let mut lt = leptess::LepTess::new(None, "eng").unwrap();
    let set_image = lt.set_image(&CAPTURE_PATH);
    if set_image.is_err() {
        panic!("Set Image Error: {:?}", set_image);
    }
    println!("Text: {}", lt.get_utf8_text().unwrap());
    
}

fn screenshot_current_url(screen: Screen, path: &str) {
    screenshot(
        screen,
        CURRENT_URL_TOP_LEFT,
        CURRENT_URL_BOTTOM_RIGHT,
        path,
    );
}

fn screenshot_captions(screen: Screen, path: &str) {
    screenshot(
        screen,
        CAPTIONS_TOP_LEFT,
        CAPTIONS_BOTTOM_RIGHT,
        path,
    );
}

fn sleep(seconds: usize) {
    let second = time::Duration::from_secs(1u64);
    thread::sleep(second * (seconds as u32));
}

fn start_stream(enigo: &mut Enigo) {
    // Windows Run
    enigo.key_down(Key::Meta);
    enigo.key_click(Key::R);
    enigo.key_up(Key::Meta);
    sleep(1);

    // Open Chrome
    enigo.key_sequence(&BROWSER_EXE);
    enigo.key_click(Key::Return);
    sleep(3);

    // Go to Hasan
    enigo.key_sequence(&HASAN_TWITCH_URL);
    enigo.key_click(Key::Return);
    sleep(5);

    // Enter Twitch's theater mode
    enigo.key_down(Key::Alt);
    enigo.key_click(Key::T);
    enigo.key_up(Key::Alt);
    sleep(5);

    // Move live captions window
    /*
    enigo.mouse_move_to(CAPTIONS_START_LOC.0, CAPTIONS_START_LOC.1);
    enigo.mouse_down(MouseButton::Left);
    enigo.mouse_move_to(CAPTIONS_END_LOC.0, CAPTIONS_END_LOC.1);
    enigo.mouse_up(MouseButton::Left);
    sleep(1);
    */

    // Move mouse off of screen
    enigo.mouse_move_to(0, 0);
    sleep(10);
}

fn screenshot(screen: Screen, top_left: (usize, usize), bottom_right: (usize, usize), path: &str) {
    screen
        .capture_area(
            top_left.0 as i32,
            top_left.1 as i32,
            (bottom_right.0 - top_left.0) as u32,
            (bottom_right.1 - top_left.1) as u32,
        )
        .unwrap()
        .save(path)
        .unwrap();
}
