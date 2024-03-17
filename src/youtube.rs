use youtube_rs::*;

use crate::video;

const YOUTUBE_UPLOAD: bool = true; // still will client, but won't actually upload
const YOUTUBE_VIDEO_DESCRIPTION: &str = "Watch Hasan at https://www.twitch.tv/hasanabi";
const YOUTUBE_VIDEO_TAGS: &str = "hasan,reacts,reactions,hasanabi,piker,react,youtube,video";

pub fn upload_to_youtube(client: &youtube_rs::YTClient, file: video::FileResult) {
    if !YOUTUBE_UPLOAD {
        return;
    }
    let video_options = VideoData {
        title: &file.name,
        desc: &YOUTUBE_VIDEO_DESCRIPTION,
        keywords: Some(&YOUTUBE_VIDEO_TAGS),
        category: youtube_rs::video::CategoryID::Entertainment as u32,
        privacy_status: youtube_rs::video::PrivacyStatus::Private, // TODO: public
        file: &file.path,
        for_kids:false,
    };
    let upload_options = client.create_upload_options(video_options).unwrap();
    client.upload_request(upload_options).expect("Could not upload");
}