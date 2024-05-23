const LLM_MODEL_BIN: &str = "D:\\Development\\models-ai\\llm\\bin\\open-llama-13b-open-instruct.ggmlv3.q4_0.bin";
const TITLE_ATTEMPTS: usize = 5;
const CAPTION_COUNT: usize = 20;

use crate::log;
use crate::text;
use crate::tools;

fn chat_request(prompt: &str, tokens: Option<usize>) -> String {
    local_llm::chat(&mut local_llm::init(LLM_MODEL_BIN), prompt, tokens)
}

fn parse_yes_no(text: &str) -> bool {
    let filtered_text = text
        .to_owned()
        .to_lowercase()
        .replace("\"", "");
    let start_text = tools::first_n_chars(&filtered_text, 8);
    let yes = start_text.contains("yes");
    let no = start_text.contains("no");
    if yes || no {
        return yes;
    }
    false
}

pub fn english_check(text: &str) -> bool {
    let prompt = "Is the following text primarily made up of English words?".to_owned()
        + " Please only reply with the word 'yes' or 'no': " + text;
    let result = chat_request(&prompt, Some(10));
    parse_yes_no(&result)
}

pub fn generate_title(captions: &Vec<String>) -> Option<String> {

    let mut all_captions = String::new();

    let length = captions.len();
    let range = std::cmp::min(CAPTION_COUNT, length);
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

    for _ in 0..TITLE_ATTEMPTS {
        let response = chat_request(&message, Some(100));
        let filtered = text::title_text_filter(&response);
        if tools::first_n_chars(&filtered.to_lowercase(), 12) == "hasan reacts" &&
            filtered.len() > 0 && filtered.len() < 100 {
            log::info(&format!("LLM gave the title: \"{}\".", &filtered));
            return Some(filtered);
        }
    }

    log::error("Could not get acceptable title from LLM attempts.");

    None
}

pub fn choose_text(text_options: &str, captions: &Vec<String>) -> String {

    let mut all_captions = String::new();

    let length = captions.len();
    let range = std::cmp::min(CAPTION_COUNT, length);
    for i in 0..range {
        all_captions = all_captions + &captions[(length / range) * i] + " ";
    }

    let message = format!(
        "I have a lot of text gathered from a video. The video
        is of a Twitch streamer reacting to a video. The text
        will include captions of both the streamer reacting
        and the video being watched itself. Here are the captions:
        {all_captions}. I also have a list of phrases to choose from,
        which is as follows: {text_options}. Based on those captions,
        and sourcing from those phrase options, which phrase would you say
        fits the video based on the captions? Reply with only the text
        of what phrase you think is best. No explaining it, no preceding it
        or posting after it, only the text itself."
    );

    chat_request(&message, Some(10))
}