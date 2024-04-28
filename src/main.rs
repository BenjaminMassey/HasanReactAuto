use enigo::*;
use screenshots::Screen;
use std::time;

mod gpt;
mod screen;
mod text;
mod thumbnail;
mod tools;
mod video;
mod youtube;

const DEBUG_MESSAGES: bool = true;
const YT_STARTUP_SECS: u64 = 60; // should match OBS replay buffer timing
const YT_FINISH_SECS: u64 = 90;
pub const SCREEN_CAP_TEMP: &str = "D:\\Development\\HRA\\screenshot.png";
pub const THUMBNAIL_TEMP: &str = "D:\\Development\\HRA\\thumbnail.png";

fn main() {
    let mut enigo = Enigo::new();
    let mut full_area = screen::CaptureArea::new();
    println!("Point to the top left of the stream and press enter.");
    full_area.top_left = tools::get_screen_point(&enigo);
    println!("Point to the bottom left of the stream and press enter.");
    full_area.bottom_right = tools::get_screen_point(&enigo);
    let url_area = screen::CaptureArea::from_percent(full_area, 0.0763f32, 0.0434f32, 0.4636f32, 0.0666f32);
    let title_area = screen::CaptureArea::from_percent(full_area, 0.006f32, 0.1165f32, 0.5551f32, 0.1543f32);
    let mut caption_area = screen::CaptureArea::new();
    println!("Point to the top left of the captions text and press enter.");
    caption_area.top_left = tools::get_screen_point(&enigo);
    println!("Point to the bottom right of the captions text and press enter.");
    caption_area.bottom_right = tools::get_screen_point(&enigo);
    let the_screen = Screen::from_point(0, 0).unwrap();
    tools::sleep(10f32);
    let mut title: Option<String> = None;
    let mut captions: Vec<String> = vec![];
    let mut yt_time: Option<time::Instant> = None;
    let mut capturing = false;
    loop {
        let url = screen::area_to_text(the_screen, url_area);
        let youtube = text::is_youtube(&url);
        if youtube && !capturing && yt_time.is_none() {
            yt_time = Some(time::Instant::now());
        } else if youtube && !capturing && yt_time.is_some()
            && yt_time.unwrap().elapsed().as_secs() > YT_STARTUP_SECS {
            video::start_capture(&mut enigo);
            screen::screenshot(the_screen, full_area, SCREEN_CAP_TEMP);
            capturing = true;
            yt_time = None;
        } else if youtube && capturing {
            yt_time = None;
            let caption = screen::area_to_text(the_screen, caption_area);
            if !caption.is_empty() {
                captions.push(caption.clone());
            }
            if title.is_none() {
                let title_area_text = text::title_text_filter(
                    &screen::area_to_text(the_screen, title_area));
                println!("title area text: {title_area_text}");
                if title_area_text.len() > 5 && gpt::gpt_english_check(&title_area_text)
                    && !text::title_text_blacklist(&title_area_text) {
                    title = Some(title_area_text)
                }
                if let Some(t) = video::try_get_title(&url) {
                    title = Some(t);
                }
            }
        } else if !youtube && !capturing {
            yt_time = None;
        } else if !youtube && capturing && yt_time.is_none() {
            yt_time = Some(time::Instant::now());
        } else if !youtube && capturing && yt_time.is_some()
            && yt_time.unwrap().elapsed().as_secs() > YT_FINISH_SECS {
            video::end_capture(&mut enigo, title.clone(), captions.clone());
            capturing = false;
            yt_time = None;
            title = None;
            captions.clear();
        } else {
            println!("Unknown scenario. T: {:?}, C: {}, Y: {}",
                yt_time, capturing, youtube,
            );
        }
        if DEBUG_MESSAGES {
            println!("\n\nDEBUG:\n\tURL: {}\n\tTitle: {:?}\n\tTime: {:?}\n\tCapturing: {}\n\tYoutube: {}\n",
                url, title, yt_time, capturing, youtube,
            );
        }
        tools::sleep(5f32);
    }
}