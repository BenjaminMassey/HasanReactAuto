use enigo::*;
use std::{thread, time};

pub fn sleep(seconds: f32) {
    let time = (1000f32 * seconds) as u64;
    let duration = time::Duration::from_millis(time);
    thread::sleep(duration);
}

pub fn get_screen_point(enigo: &Enigo) -> (i32, i32) {
    std::io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read line");
    enigo.mouse_location()
}

pub fn keyboard_command(enigo: &mut Enigo, held: &[Key], click: Key) {
    for &key in held {
        enigo.key_down(key);
        sleep(0.1f32);
    }
    sleep(0.3f32);
    enigo.key_click(click);
    for &key in held {
        enigo.key_up(key);
        sleep(0.1f32);
    }
}