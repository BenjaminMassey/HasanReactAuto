use raster::filter;
use screenshots::Screen;
use std::fmt;

const CAPTURE_PATH: &str = "./capture.png"; // for temp screenshots

#[derive(Clone, Copy)]
pub struct CaptureArea {
    pub top_left: (i32, i32),
    pub bottom_right: (i32, i32),
}

impl CaptureArea {
    pub fn new() -> Self {
        CaptureArea {
            top_left: (0, 0),
            bottom_right: (0, 0),
        }
    }
    pub fn from_percent(original: CaptureArea, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let orig_size = (
            (original.bottom_right.0 - original.top_left.0) as f32,
            (original.bottom_right.1 - original.top_left.1) as f32,
        );
        CaptureArea{
            top_left: (
                (orig_size.0 * x1) as i32 + original.top_left.0,
                (orig_size.1 * y1) as i32 + original.top_left.1,
            ),
            bottom_right: (
                (orig_size.0 * x2) as i32 + original.top_left.0,
                (orig_size.1 * y2) as i32 + original.top_left.1,
            ),
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

pub fn screenshot(screen: Screen, area: CaptureArea, path: &str) {
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

pub fn area_to_text(screen: Screen, area: CaptureArea) -> String {
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