use enigo::*;
use glob::glob;
use raster::filter;
use screenshots::Screen;
use std::{f32::consts::E, fmt, io, thread, time};

const DEBUG_MESSAGES: bool = true;
const CAPTURE_PATH: &str = "./capture.png";
const VIDEOS_PATH: &str = "C:/Users/benja/Videos/";
const VIDEO_EXT: &str = ".mp4";

#[derive(Clone, Copy)]
struct CaptureArea {
    top_left: (i32, i32),
    bottom_right: (i32, i32),
}

impl CaptureArea {
    fn new() -> Self {
        CaptureArea {
            top_left: (0, 0),
            bottom_right: (0, 0),
        }
    }
}
impl fmt::Display for CaptureArea {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({}, {}) => ({}, {})", 
            self.top_left.0,
            self.top_left.1,
            self.bottom_right.0,
            self.bottom_right.1,
        )
    }
}

fn main() {
    let mut enigo = Enigo::new();
    let mut url_area = CaptureArea::new();
    url_area.top_left = get_screen_point(&enigo);
    url_area.bottom_right = get_screen_point(&enigo);
    let mut caption_area = CaptureArea::new();
    caption_area.top_left = get_screen_point(&enigo);
    caption_area.bottom_right = get_screen_point(&enigo);
    let screen = Screen::from_point(0, 0).unwrap();
    sleep(10);
    let mut title: Option<String> = None;
    let mut captions: Vec<String> = vec![];
    let mut yt_time: Option<time::Instant> = None;
    let mut capturing = false;
    loop {
        let caption = area_to_text(screen, caption_area);
        if caption.len() > 0 {
            captions.push(caption.clone());
        }
        let url = area_to_text(screen, url_area);
        let youtube = is_youtube(&url);
        if youtube && !capturing && yt_time.is_none() {
            yt_time = Some(time::Instant::now());
        } else if youtube && !capturing && yt_time.is_some()
            && yt_time.unwrap().elapsed().as_secs() > 10u64 {
            start_capture(&mut enigo);
            capturing = true;
            yt_time = None;
        } else if youtube && capturing {
            yt_time = None;
            if let Some(t) = try_get_title(&url) {
                title = Some(t);
            }
        } else if !youtube && !capturing {
            yt_time = None;
        } else if !youtube && capturing && yt_time.is_none() {
            yt_time = Some(time::Instant::now());
        } else if !youtube && capturing && yt_time.is_some()
            && yt_time.unwrap().elapsed().as_secs() > 30u64 {
            end_capture(&mut enigo, title.clone());
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
            println!("\n\nText:\n\tURL: {}\n\tCaption: {}\n\tTitle: {:?}",
                url, caption, title,
            );
            println!("State:\n\tTime: {:?}\n\tCapturing: {}\n\tYoutube: {}",
                yt_time, capturing, youtube,
            );
        }
        sleep(5);
    }
}

fn sleep(seconds: usize) {
    let second = time::Duration::from_secs(1u64);
    thread::sleep(second * (seconds as u32));
}

fn get_screen_point(enigo: &Enigo) -> (i32, i32) {
    io::stdin()
        .read_line(&mut String::new())
        .expect("Failed to read line");
    enigo.mouse_location()
} 

fn screenshot(screen: Screen, area: CaptureArea, path: &str) {
    screen
        .capture_area(
            area.top_left.0,
            area.top_left.1,
            (area.bottom_right.0 - area.top_left.0) as u32,
            (area.bottom_right.1 - area.top_left.1) as u32,
        )
        .unwrap()
        .save(path)
        .unwrap();
}

fn area_to_text(screen: Screen, area: CaptureArea) -> String {
    screenshot(
        screen,
        area,
        &CAPTURE_PATH,
    );
    let mut image = raster::open(&CAPTURE_PATH).unwrap();
    filter::sharpen(&mut image).unwrap();
    raster::save(&image, &CAPTURE_PATH).unwrap();
    let mut leptess = leptess::LepTess::new(None, "eng").unwrap();
    let set_image = leptess.set_image(&CAPTURE_PATH);
    if set_image.is_err() {
        println!("Set Image Error: {:?}", set_image);
        return "".to_owned();
    }
    let text = leptess.get_utf8_text();
    if text.is_err() {
        println!("Get Text Error: {:?}", text);
        return "".to_owned();
    }
    text.unwrap()
}

fn is_youtube(text: &str) -> bool {
    let the_text = screen_text_filter(&text.to_owned().to_lowercase());
    the_text.contains("outube") ||
        the_text.contains("autube") ||
        the_text.contains("ouuube") ||
        the_text.contains("ouiube") ||
        the_text.contains("oulube")
}

fn start_capture(enigo: &mut Enigo) {
    enigo.key_down(Key::Control);
    enigo.key_down(Key::Alt);
    sleep(1);
    enigo.key_click(Key::F6);
    enigo.key_up(Key::Control);
    enigo.key_up(Key::Alt);
}

fn end_capture(enigo: &mut Enigo, title: Option<String>) {
    enigo.key_down(Key::Control);
    enigo.key_down(Key::Alt);
    sleep(1);
    enigo.key_click(Key::F7);
    enigo.key_up(Key::Control);
    enigo.key_up(Key::Alt);
    sleep(5);
    if title.is_some() {
        update_title(&title.unwrap());
    }
}

fn screen_text_filter(text: &str) -> String {
    text
        .to_owned()
        .replace(" ", "")
        .replace("|", "I")
        .replace("‘", "")
        .replace("`", "")
        .replace(".comy", ".com/")
        .replace("watchv=", "watch?v=")
        .replace("becom", "be.com") // youtubecom => youtube.com
        .replace("comwatch", "com/watch")
    // TODO: more filtering conditions
}

fn try_get_title(yt_url: &str) -> Option<String> {
    let noembed_url = "https://noembed.com/embed?url=https://".to_owned() + &screen_text_filter(yt_url);
    let result = reqwest::blocking::get(noembed_url);
    if result.is_err() {
        return None;
    }
    let json = serde_json::from_str(&result.unwrap().text().unwrap());
    if json.is_err() {
        return None;
    }
    let value: serde_json::Value = json.unwrap();
    let title = value.get("title");
    if title.is_none() {
        return None;
    }
    Some(title.unwrap().to_string())
}

fn update_title(title: &str) {
    let vid_paths = glob(&(VIDEOS_PATH.to_owned() + "*" + &VIDEO_EXT))
        .unwrap()
        .filter_map(std::result::Result::ok);
    let mut vids = vid_paths
        .map(|p| p.into_os_string().into_string().unwrap())
        .collect::<Vec<String>>();
    vids.sort_by(|a, b| a.to_string().to_lowercase().cmp(&b.to_lowercase()));
    let possible_vid = vids.last();
    if let Some(vid) = possible_vid {
        std::fs::rename(
            vid, &(VIDEOS_PATH.to_owned() + title + &VIDEO_EXT)
        ).unwrap();
    }
}