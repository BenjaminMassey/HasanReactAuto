use enigo::*;
use glob::glob;
use raster::filter;
use screenshots::Screen;
use std::{fmt, io, thread, time};

const DEBUG_MESSAGES: bool = true;
const CAPTURE_PATH: &str = "./capture.png";
const IN_VIDEOS_PATH: &str = "C:\\Users\\benja\\Videos\\";
const OUT_VIDEOS_PATH: &str = "C:\\Users\\benja\\Videos\\HRA\\";
const VIDEO_EXT: &str = ".mp4";
const START_REC_HELD: &[Key] = &[Key::Control, Key::Alt];
const START_REC_CLICK: Key = Key::F6;
const STOP_REC_HELD: &[Key] = &[Key::Control, Key::Alt];
const STOP_REC_CLICK: Key = Key::F7;
const GPT_URL: &str = "127.0.0.1:4891";
const GPT_MODEL: &str = "Nous Hermes 2 Mistral DPO";
const GPT_ATTEMPTS: usize = 5;

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
    sleep(10f32);
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
            end_capture(&mut enigo, title.clone(), captions.clone());
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
        sleep(5f32);
    }
}

fn sleep(seconds: f32) {
    let time = (1000f32 * seconds) as u64;
    let duration = time::Duration::from_millis(time);
    thread::sleep(duration);
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

fn keyboard_command(enigo: &mut Enigo, held: &[Key], click: Key) {
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

fn start_capture(enigo: &mut Enigo) {
    keyboard_command(enigo, START_REC_HELD, START_REC_CLICK);
}

fn end_capture(enigo: &mut Enigo, title: Option<String>, captions: Vec<String>) {
    keyboard_command(enigo, STOP_REC_HELD, STOP_REC_CLICK);
    sleep(5f32);
    update_title(title, captions);
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

fn path_clean(path: &str) -> String {
    path
        .to_owned()
        .replace("\"", "")
        .replace("\\", "")
        .replace("/", "")
        .replace(".", "")
}

fn update_title(title: Option<String>, captions: Vec<String>) {
    let vid_paths = glob(&(IN_VIDEOS_PATH.to_owned() + "*" + &VIDEO_EXT))
        .unwrap()
        .filter_map(std::result::Result::ok);
    let mut vids = vid_paths
        .map(|p| p.into_os_string().into_string().unwrap())
        .collect::<Vec<String>>();
    vids.sort_by(|a, b| a.to_string().to_lowercase().cmp(&b.to_lowercase()));
    let possible_vid = vids.last();
    let the_title: Option<String> = {
        if let Some(retrieved) = title {
            Some("Hasan reacts to ".to_owned() + &retrieved)
        } else {
            let mut result = None;
            for _ in 0..GPT_ATTEMPTS {
                let answer = gpt_title(&captions);
                if let Some(text) = answer {
                    if text.len() > 0 && text.len() < 50 {
                        result = Some(text.clone());
                        break;
                    }
                }
            }
            result
        }
    };
    if let Some(final_title) = the_title {
        if let Some(source) = possible_vid {
            let destination = OUT_VIDEOS_PATH.to_owned()
                + &path_clean(&final_title) + &VIDEO_EXT;
            let rename = std::fs::rename(source, &destination);
            if rename.is_err() {
                println!("Rename failed. File: {source}");
            } else {
                println!("Rename success. File: {destination}");
            }
        }
    }
}

fn local_gpt_body(message: &str) -> String {
    format!(
        r#"
        {{
            "model": "{GPT_MODEL}",
            "max_tokens": 100,
            "messages": [
                {{
                    "role": "system",
                    "content": "You are a helpful assistant."
                }},
                {{
                    "role": "user",
                    "content": "{message}"
                }}
            ]
        }}
        "#
    )
}

fn local_gpt_chat(message: &str) -> Option<String> {
    let url = "http://".to_owned() + &GPT_URL + "/v1/chat/completions";
    let client = reqwest::blocking::Client::new();
    let body = local_gpt_body(message);
    let result = client.post(url).body(body).send();
    if result.is_err() {
        return None;
    }
    let json = serde_json::from_str(&result.unwrap().text().unwrap());
    if json.is_err() {
        return None;
    }
    let value: serde_json::Value = json.unwrap();
    let choices = value.get("choices");
    if choices.is_none() {
        return None;
    }
    let message = choices.unwrap()[0].get("message");
    if message.is_none() {
        return None;
    }
    let content = message.unwrap().get("content");
    if content.is_none() {
        return None;
    }
    Some(content.unwrap().to_string())
}

fn gpt_title(captions: &Vec<String>) -> Option<String> {

    let mut all_captions = String::new();

    let length = captions.len();
    let range = std::cmp::min(50, length);
    for i in 0..range {
        all_captions = all_captions + &captions[(length / range) * i] + " ";
    }

    let message = format!(
        "I have a lot of text gathered from a video. The video
        is of a Twitch streamer reacting to a video. The text
        will include captions of both the streamer reacting
        and the video being watched itself. Here are the captions:
        {all_captions}. Based on those captions, what short and 
        what would you guess is the title of video that is being
        reacted to? Reply with only the text 'Hasan reacts to' followed
        by your video title guess."
    );

    local_gpt_chat(&message)
}