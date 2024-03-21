use enigo::*;

use crate::video;
use crate::tools;

const YOUTUBE_UPLOAD: bool = true; // still will client, but won't actually upload
const YOUTUBE_PUBLIC: bool = false;
const YOUTUBE_VIDEO_DESCRIPTION: &str = "Watch Hasan on Twitch at hasanabi";

pub fn upload_to_youtube(enigo: &mut Enigo, file: video::FileResult) {
    if !YOUTUBE_UPLOAD {
        return;
    }
    tools::keyboard_command(enigo, &[Key::Control], Key::T); // open a new tab
    enigo.key_sequence("youtube.com"); // enter url
    tools::click_and_pause(enigo, Key::Return); // go to url
    tools::sleep(2.5f32);           
    for _ in 0..7 {
        tools::click_and_pause(enigo, Key::Tab); // select 'create'
    }
    tools::click_and_pause(enigo, Key::Return); // accept 'create'
    tools::click_and_pause(enigo, Key::Return); // accept 'upload video'
    tools::sleep(5f32);
    for _ in 0..3 {
        tools::click_and_pause(enigo, Key::Tab); // select 'select files'
    }
    tools::click_and_pause(enigo, Key::Return); // accept 'select files'
    tools::sleep(1f32);
    enigo.key_sequence(&file.path); // type out full video path
    tools::click_and_pause(enigo, Key::Return); // accept path
    tools::sleep(10f32);
    // TODO: this feels potentially better, because it means the title
        //   can be as custom as wanted, but youtube limits to 100 char
        //   title, so abusing its auto-titling for now
    //tools::keyboard_command(enigo, &[Key::Control], Key::A); // select title text
    //tools::click_and_pause(enigo, Key::Backspace); // delete default title
    //enigo.key_sequence(&file.name); // type out title
    for _ in 0..2 {
        tools::click_and_pause(enigo, Key::Tab); // select description box
    }
    enigo.key_sequence(YOUTUBE_VIDEO_DESCRIPTION); // type out description
    for _ in 0..2 {
        tools::click_and_pause(enigo, Key::Tab); // select 'upload thumbnail'
    }
    // TODO: need to make thumbnails and add to FileResult, first
    //tools::click_and_pause(enigo, Key::Return); // accept 'upload thumbnail'
    //tools::sleep(1f32);
    //enigo.key_sequence(&file.thumbnail); // type out full image path
    //tools::click_and_pause(enigo, Key::Return); // accept path
    //tools::sleep(6f32);
    for _ in 0..6 {
        tools::click_and_pause(enigo, Key::Tab); // select kids selection
    }
    tools::sleep(0.5f32);
    tools::click_and_pause(enigo, Key::DownArrow); // select no
    for _ in 0..5 {
        tools::click_and_pause(enigo, Key::Tab); // select next
    }
    tools::click_and_pause(enigo, Key::Return); // accept next
    for _ in 0..12 {
        tools::click_and_pause(enigo, Key::Tab); // select next
    }
    tools::click_and_pause(enigo, Key::Return); // accept next
    for _ in 0..11 {
        tools::click_and_pause(enigo, Key::Tab); // select next
    }
    tools::click_and_pause(enigo, Key::Return); // accept next
    for _ in 0..12 {
        tools::click_and_pause(enigo, Key::Tab); // select visibility
    }
    for _ in 0..2 {
        tools::click_and_pause(enigo, Key::DownArrow); // select public
    }
    if !YOUTUBE_PUBLIC {
        for _ in 0..2 {
            tools::click_and_pause(enigo, Key::UpArrow); // select private
        }
    }
    for _ in 0..9 {
        tools::click_and_pause(enigo, Key::Tab); // select publish
    }
    tools::click_and_pause(enigo, Key::Return); // accept publish
    tools::sleep(1f32);
    tools::keyboard_command(enigo, &[Key::Control], Key::Num1); // back to hasan tab
}