use enigo::*;

use crate::llm;
use crate::log;
use crate::text;
use crate::thumbnail;
use crate::tools;
use crate::youtube;

const IN_VIDEOS_PATH: &str = "D:\\OBS\\Recordings\\";
const OUT_VIDEOS_PATH: &str = "D:\\Development\\HRA\\";
const VIDEO_EXT: &str = ".mp4";
const START_REC_HELD: &[Key] = &[Key::Control, Key::Alt];
const START_REC_CLICK: Key = Key::F6; // SHOULD ALSO BE REPLAY BUFFER SAVE
const STOP_REC_HELD: &[Key] = &[Key::Control, Key::Alt];
const STOP_REC_CLICK: Key = Key::F7;
const MERGE_ATTEMPTS: usize = 3;

pub fn start_capture(enigo: &mut Enigo) {
    println!("\n\n=== START CAPTURE ===\n\n");
    tools::keyboard_command(enigo, START_REC_HELD, START_REC_CLICK);
}

pub fn end_capture(enigo: &mut Enigo, title: Option<String>, captions: Vec<String>) {
    println!("\n\n=== END CAPTURE ===\n\n");
    tools::keyboard_command(enigo, STOP_REC_HELD, STOP_REC_CLICK);
    tools::sleep(5f32);
    let result = update_title(title, &captions);
    if let Some(file) = result {
        thumbnail::generate(crate::SCREEN_CAP_TEMP, crate::THUMBNAIL_TEMP, &captions);
        youtube::upload_to_youtube(enigo, file); // TODO: async
    } else {
        log::error("No proper video file result: failure to even start upload attempt.");
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

pub fn _test_recent() -> (String, String) {
    let res = recent_video();
    (res.replay.unwrap(), res.recording.unwrap())
}

fn combine_glob_result(result: &GlobResult) -> Option<String> {
    if result.replay.is_none() || result.recording.is_none() {
        log::error("Replay or recording was None when merge was attempted: aborted.");
        return None;
    }
    let files = [&result.replay.as_ref().unwrap(), &result.recording.as_ref().unwrap()];
    let out_file = {
        if let Ok(time) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            OUT_VIDEOS_PATH.to_owned() + &time.as_secs().to_string() + "output.mp4"
        } else {
            OUT_VIDEOS_PATH.to_owned() + "output.mp4"
        }
    };
    let merge = mp4_merge::join_files(
        &files,
        &&&out_file, // three &s???
        |progress| {
            println!("mp4 merging... {:.2}%", progress * 100.0);
        }
    );
    if merge.is_ok() {
        return Some(out_file);
    }
    log::error(&format!("Merge failed: {:?}", merge));
    None
}

fn try_combine_glob_result(result: GlobResult) -> Option<String> {
    for _ in 0..MERGE_ATTEMPTS {
        let gr = combine_glob_result(&result);
        if gr.as_ref().is_some() && mp4_duration(&gr.as_ref().unwrap()) > 0 {
            return gr;
        }
        log::warning("MP4 merge failed, trying again...");
    }
    log::error(&format!("MP4 merge failed {} times: aborting.", MERGE_ATTEMPTS));
    None
}

pub fn _test_combine(file_1: &str, file_2: &str) -> Option<String> {
    let gr = GlobResult {
        replay: Some(file_1.to_owned()),
        recording: Some(file_2.to_owned()),
    };
    try_combine_glob_result(gr)
}

pub fn update_title(title: Option<String>, captions: &Vec<String>) -> Option<FileResult> {
    let source_file = try_combine_glob_result(recent_video())?;
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
            println!("Failed to get real title: trying LLM.");
            llm::generate_title(&captions)
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

pub fn mp4_duration(path: &str) -> u64 {
    let attempt = std::fs::File::open(path);
    if attempt.is_err() {
        return 0;
    }
    let file = attempt.unwrap();
    let meta = file.metadata();
    if meta.is_err() {
        return 0;
    }
    let size = meta.unwrap().len();
    let reader = std::io::BufReader::new(file);
    let mp4 = mp4::Mp4Reader::read_header(reader, size);
    if mp4.is_err() {
        return 0;
    }
    mp4.unwrap().duration().as_secs()
}