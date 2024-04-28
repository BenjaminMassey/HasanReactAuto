const GPT_URL: &str = "127.0.0.1:4891";
const GPT_MODEL: &str = "Nous Hermes 2 Mistral DPO";

pub fn local_gpt_body(message: &str, tokens: usize) -> String {
    format!(
        r#"
        {{
            "model": "{GPT_MODEL}",
            "max_tokens": {tokens},
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

pub fn local_gpt_chat(message: &str, tokens: usize) -> Option<String> {
    let url = "http://".to_owned() + &GPT_URL + "/v1/chat/completions";
    let client = reqwest::blocking::Client::new();
    let body = local_gpt_body(message, tokens);
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

pub fn gpt_yes_no(text: &str) -> bool {
    let filtered_text = text
        .to_owned()
        .to_lowercase()
        .replace("\"", "");
    let start_text = &filtered_text[..std::cmp::min(filtered_text.len(), 8)];
    let yes = start_text.contains("yes");
    let no = start_text.contains("no");
    if yes || no {
        return yes;
    }
    false
}

pub fn gpt_english_check(text: &str) -> bool {
    let prompt = "Is the following text primarily made up of English words?".to_owned()
        + " Please only reply with the word 'yes' or 'no': " + text;
    let result = local_gpt_chat(&prompt, 10);
    println!("GPT English Check: {} => {:?}", text, result);
    if let Some(answer) = result {
        gpt_yes_no(&answer)
    } else {
        false
    }
}

pub fn gpt_title(captions: &Vec<String>) -> Option<String> {

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

    local_gpt_chat(&message, 100)
}

pub fn gpt_text(text_options: &str, captions: &Vec<String>) -> Option<String> {

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
        {all_captions}. I also have a list of phrases to choose from,
        which is as follows: {text_options}. Based on those captions,
        and sourcing from those phrase options, which phrase would you say
        fits the video based on the captions? Reply with only the text
        of what phrase you think is best. No explaining it, no preceding it
        or posting after it, only the text itself."
    );

    local_gpt_chat(&message, 10)
}