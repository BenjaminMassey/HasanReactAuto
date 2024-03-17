use enigo::*;
use glob::glob;
use youtube_rs::YTClient;

use crate::gpt;
use crate::text;
use crate::tools;
use crate::youtube;

const IN_VIDEOS_PATH: &str = "C:\\Users\\benja\\Videos\\";
const OUT_VIDEOS_PATH: &str = "C:\\Users\\benja\\Videos\\HRA\\";
const VIDEO_EXT: &str = ".mp4";
const START_REC_HELD: &[Key] = &[Key::Control, Key::Alt];
const START_REC_CLICK: Key = Key::F6;
const STOP_REC_HELD: &[Key] = &[Key::Control, Key::Alt];
const STOP_REC_CLICK: Key = Key::F7;
const GPT_ATTEMPTS: usize = 5; // for final backup title generation

pub fn start_capture(enigo: &mut Enigo) {
    println!("\n\n=== START CAPTURE ===\n\n");
    tools::keyboard_command(enigo, START_REC_HELD, START_REC_CLICK);
}

pub fn end_capture(client: &YTClient, enigo: &mut Enigo, title: Option<String>, captions: Vec<String>) {
    println!("\n\n=== END CAPTURE ===\n\n");
    tools::keyboard_command(enigo, STOP_REC_HELD, STOP_REC_CLICK);
    tools::sleep(5f32);
    let result = update_title(title, captions);
    if let Some(file) = result {
        youtube::upload_to_youtube(&client, file); // TODO: async
    }
}


pub fn try_get_title(yt_url: &str) -> Option<String> {
    let noembed_url = "https://noembed.com/embed?url=https://".to_owned()
        + &text::youtube_url_filter(yt_url);
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

pub struct FileResult {
    pub path: String,
    pub name: String,
}

pub fn update_title(title: Option<String>, captions: Vec<String>) -> Option<FileResult> {
    let vid_paths = glob(&(IN_VIDEOS_PATH.to_owned() + "*" + &VIDEO_EXT))
        .unwrap()
        .filter_map(std::result::Result::ok);
    let mut vids = vid_paths
        .map(|p| p.into_os_string().into_string().unwrap())
        .collect::<Vec<String>>();
    vids.sort_by(|a, b| a.to_string().to_lowercase().cmp(&b.to_lowercase()));
    let possible_vid = vids.last();
    let gathered_title: Option<String> = {
        if let Some(retrieved) = title {
            if gpt::gpt_english_check(&retrieved) {
                Some("Hasan reacts to ".to_owned() + &retrieved)
            } else {
                None
            }
        }
        else {
            None
        }
    };
    let generated_title: Option<String> = {
        if gathered_title.is_none() {
            println!("Failed to get real title.");
            let mut result = None;
            for _ in 0..GPT_ATTEMPTS {
                println!("Trying GPT...");
                let answer = gpt::gpt_title(&captions);
                if let Some(text) = answer {
                    if text.len() > 0 && text.len() < 50 {
                        result = Some(text.clone());
                        break;
                    }
                }
            }
            result
        } else {
            None
        }
    };
    let final_title: Option<String> = {
        if let Some(gt) = gathered_title {
            Some(gt)
        } else if let Some(gt) = generated_title {
            Some(gt)
        } else {
            None
        }
    };
    if let Some(the_title) = final_title {
        if let Some(source) = possible_vid {
            // TODO: file rename kind of unneeded, since for youtube upload, and path + title separate...
            let destination = OUT_VIDEOS_PATH.to_owned()
                + &text::path_clean(&the_title) + &VIDEO_EXT;
            let rename = std::fs::rename(source, &destination);
            if rename.is_err() {
                println!("\n\nRename failed. File: {source} => {destination}\n\n");
                return Some(FileResult{path: source.to_owned(), name: the_title});
            } else {
                println!("\n\nRename success. File: {destination}\n\n");
                return Some(FileResult{path: destination, name: the_title});
            }
        }
    }
    println!("\n\n===============\n\nNo title was achieved.\n\n===============\n\n");  
    None
}