use enigo::*;
use screenshots::Screen;
use std::time;

mod llm;
mod log;
mod screen;
mod text;
mod thumbnail;
mod tools;
mod video;
mod youtube;

const DEBUG_MESSAGES: bool = true;
const YT_STARTUP_SECS: u64 = 90; // should match OBS replay buffer timing
const YT_FINISH_SECS: u64 = 120;
pub const SCREEN_CAP_TEMP: &str = "D:\\Development\\HRA\\screenshot.png";
pub const THUMBNAIL_TEMP: &str = "D:\\Development\\HRA\\thumbnail.png";

fn main() {
    println!("Choose Text: {} (poop)",
        llm::choose_text(
            "poop, clean, perfect",
            &vec!("the worst".to_owned(), "garbage everywhere".to_owned(),
                "feces-ridden".to_owned()))
    );
    println!("English Checks: {}, {} (false, true)",
        llm::english_check("foijwijfwiejw fweifjow ioo8130 138sfj"),
        llm::english_check("the fox is a big bad meanie")
    );
    println!("Generate Title: {:?} (Some(something anti-israel related))",
        llm::generate_title(
            &vec!(
                "israel committing more genocide".to_owned(),
                "the worst crime to palenstine since the nakbah".to_owned(),
                "the us is complicit in israel's ethnic cleansing".to_owned(),
            )
        )
    );
    panic!();
    log::info("Starting application.");
    let mut enigo = Enigo::new();
    let mut full_area = screen::CaptureArea::new();
    println!("Point to the top left of the stream and press enter.");
    full_area.top_left = tools::get_screen_point(&enigo);
    println!("Point to the bottom right of the stream and press enter.");
    full_area.bottom_right = tools::get_screen_point(&enigo);
    let url_area = screen::CaptureArea::from_percent(full_area, 0.0763f32, 0.0434f32, 0.4636f32, 0.0666f32);
    let title_area_1 = screen::CaptureArea::from_percent(full_area, 0.006f32, 0.1165f32, 0.5551f32, 0.1543f32);
    let title_area_2 = screen::CaptureArea::from_percent(full_area, 0.006f32, 0.8650f32, 0.4900f32, 0.9000f32);
    log::info("Main screen capture area is setup.");
    let mut caption_area = screen::CaptureArea::new();
    println!("Point to the top left of the captions text and press enter.");
    caption_area.top_left = tools::get_screen_point(&enigo);
    println!("Point to the bottom right of the captions text and press enter.");
    caption_area.bottom_right = tools::get_screen_point(&enigo);
    log::info("Captions text capture area is setup.");
    let the_screen = Screen::from_point(0, 0).unwrap();
    println!("Pausing for ten seconds: click back into Chrome.");
    tools::sleep(10f32);
    let mut title: Option<String> = None;
    let mut captions: Vec<String> = vec![];
    let mut yt_time: Option<time::Instant> = None;
    let mut capturing = false;
    log::info("Core loop starting.");
    loop {
        let url = screen::area_to_text(the_screen, url_area);
        let youtube = text::is_youtube(&url);
        if youtube && !capturing && yt_time.is_none() {
            yt_time = Some(time::Instant::now());
        } else if youtube && !capturing && yt_time.is_some()
            && yt_time.unwrap().elapsed().as_secs() > YT_STARTUP_SECS {
            log::info("Starting a capture.");
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
                for title_area in vec![title_area_1, title_area_2] {
                    if title.is_some() {
                        break;
                    }
                    let title_area_text = text::title_text_filter(
                        &screen::area_to_text(the_screen, title_area));
                    if title_area_text.len() > 5 && llm::english_check(&title_area_text)
                        && !text::title_text_blacklist(&title_area_text) {
                        log::info(&format!("Got title from screen area: {}", &title_area_text));
                        title = Some(title_area_text)
                    }
                }
                if title.is_none() {
                    if let Some(t) = video::try_get_title(&url) {
                        log::info(&format!("Got title from URL: {}", &t));
                        title = Some(t);
                    }
                }
                if let Some(t) = &title {
                    log::info(&format!("Set title to \"{}\".", t));
                }
            }
        } else if !youtube && !capturing {
            yt_time = None;
        } else if !youtube && capturing && yt_time.is_none() {
            yt_time = Some(time::Instant::now());
        } else if !youtube && capturing && yt_time.is_some()
            && yt_time.unwrap().elapsed().as_secs() > YT_FINISH_SECS {
            log::info("Ending capture.");
            video::end_capture(&mut enigo, title.clone(), captions.clone());
            capturing = false;
            yt_time = None;
            title = None;
            captions.clear();
        } else if yt_time.is_some() && !capturing && youtube {
            println!("Still YouTube video, waiting for time threshold...");
        } else if yt_time.is_some() && capturing && !youtube {
            println!("YouTube video appears to be over, waiting for time threshold...");
        } else {
            log::warning(
                &format!(
                    "Hit an unknown scenario. yt_time: {:?}, capturing: {}, youtube: {}",
                    yt_time,
                    capturing,
                    youtube
                )
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