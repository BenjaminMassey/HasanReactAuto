pub fn is_youtube(text: &str) -> bool {
    let the_text = youtube_url_filter(&text.to_owned().to_lowercase());
    the_text.contains("outube") ||
        the_text.contains("autube") ||
        the_text.contains("ouuube") ||
        the_text.contains("ouiube") ||
        the_text.contains("oulube")
}

pub fn title_text_blacklist(text: &str) -> bool {
    text.contains("(%")
        || text.contains("(¢")
        || text.contains("(&")
        || text.contains("Search Q")
        || text.contains("Premium Search")
    // Top bar of youtube, where it says "Premium" and has the search bar
}

pub fn youtube_url_filter(text: &str) -> String {
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

pub fn title_text_filter(text: &str) -> String {
    text
        .to_owned()
        .replace("|", "I")
        .replace("‘", "'")
        .replace("`", "'")
        .replace("\n", "")
    // TODO: more filtering conditions
}

pub fn path_clean(path: &str) -> String {
    path
        .to_owned()
        .replace(".", "")
        .replace("/", "")
        .replace("\\", "")
        .replace(":", "")
        .replace("*", "")
        .replace("?", "")
        .replace("\"", "'")
        .replace("<", "")
        .replace(">", "")
        .replace("|", "")
}