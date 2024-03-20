use enigo::*;
use youtube_rs::YTClient;

use crate::gpt;
use crate::text;
use crate::tools;
use crate::youtube;

const IN_VIDEOS_PATH: &str = "D:\\OBS\\Recordings\\";
const OUT_VIDEOS_PATH: &str = "D:\\Development\\HRA\\";
const VIDEO_EXT: &str = ".mp4";
const START_REC_HELD: &[Key] = &[Key::Control, Key::Alt];
const START_REC_CLICK: Key = Key::F6; // SHOULD ALSO BE REPLAY BUFFER SAVE
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

struct GlobResult {
    replay: Option<String>,
    recording: Option<String>,
}
impl GlobResult {
    fn new() -> Self {
        Self { replay: None, recording: None, }
    }
}

pub struct FileResult {
    pub path: String,
    pub name: String,
}

fn recent_video() -> GlobResult {
    let glob_params = vec![
        IN_VIDEOS_PATH.to_owned() + "[!{Replay}]*" + VIDEO_EXT,
        IN_VIDEOS_PATH.to_owned() + "Replay*" + VIDEO_EXT,
    ];
    let mut result = GlobResult::new();
    for (i, glob_param) in glob_params.iter().enumerate() {
        let vid_paths = glob::glob(glob_param)
            .unwrap()
            .filter_map(std::result::Result::ok);
        let mut vids = vid_paths
            .map(|p| p.into_os_string().into_string().unwrap())
            .collect::<Vec<String>>();
        vids.sort_by(|a, b| a.to_string().to_lowercase().cmp(&b.to_lowercase()));
        if i == 0 { // TODO: hate this
            result.recording = vids.last().cloned();
        } else {
            result.replay = vids.last().cloned();
        }
    }
    result
}

fn combine_glob_result(result: GlobResult) -> String {
    let files = [&result.replay.unwrap(), &result.recording.unwrap()];
    let out_file = {
        if let Ok(time) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            OUT_VIDEOS_PATH.to_owned() + &time.as_secs().to_string() + "output.mp4"
        } else {
            OUT_VIDEOS_PATH.to_owned() + "output.mp4"
        }
    };
    mp4_merge::join_files(
        &files,
        &&out_file,
        |progress| {
            println!("mp4 merging... {:.2}%", progress * 100.0);
        }
    ).unwrap();
    out_file
}

pub fn update_title(title: Option<String>, captions: Vec<String>) -> Option<FileResult> {
    let source_file = combine_glob_result(recent_video());
    let gathered_title: Option<String> = {
        if let Some(retrieved) = title {
            Some("Hasan reacts to ".to_owned() + &retrieved)
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
                    println!("GPT title: {text}");
                    if text.len() > 0 && text.len() < 100 {
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
        // TODO: file rename kind of unneeded, since for youtube upload, and path + title separate...
        let destination = OUT_VIDEOS_PATH.to_owned()
            + &text::path_clean(&the_title) + VIDEO_EXT;
        let rename = std::fs::rename(&source_file, &destination);
        if rename.is_err() {
            println!("\n\nRename failed. File: {} => {}\n\n", &source_file, &destination);
            return Some(FileResult{path: source_file, name: the_title});
        } else {
            println!("\n\nRename success. File: {destination}\n\n");
            return Some(FileResult{path: destination, name: the_title});
        }
    }
    println!("\n\n===============\n\nNo title was achieved.\n\n===============\n\n");  
    None
}