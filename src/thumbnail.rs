use rand::seq::SliceRandom;
use rand::thread_rng;
use raster::{editor, BlendMode, PositionMode, ResizeMode};
use std::io::Write;
use text_to_png::TextRenderer;

const FONT_TTF_BYTES: &[u8] = include_bytes!("D:\\Downloads\\Roboto\\Roboto-Bold.ttf");
const HASAN_FACES_DIR: &str = "C:\\Users\\benja\\Pictures\\Hasan-Face\\Final\\";
const TEXT_OPTIONS_FILE: &str = "D:\\Development\\HRA\\thumbnail_texts.txt";

pub fn generate(background_file: &str, result_file: &str) {
    let renderer =
        TextRenderer::try_new_with_ttf_font_data(FONT_TTF_BYTES).expect("Failed to load font");

    let text_options_raw = std::fs::read_to_string(TEXT_OPTIONS_FILE).expect("Failed to open text options file");

    let mut text_options: Vec<String> = text_options_raw.split("\n").map(|s| s.to_owned()).collect();
    text_options.shuffle(&mut thread_rng());

    let text_png = renderer
        .render_text_to_png_data(text_options.first().unwrap(), 256, 0xFF0000)
        .expect("Failed to text_to_png");

    let mut text_file = std::fs::File::options()
        .create(true)
        .write(true)
        .open("text.png")
        .expect("Couldn't make or write text.png");
    let _ = text_file.write_all(&text_png.data);
    let _ = text_file.flush();

    let hasan_face_paths = glob::glob(&(HASAN_FACES_DIR.to_owned() + "*.png"))
        .unwrap()
        .filter_map(std::result::Result::ok);
    let mut hasan_faces = hasan_face_paths
        .map(|p| p.into_os_string().into_string().unwrap())
        .collect::<Vec<String>>();
    hasan_faces.shuffle(&mut thread_rng());
    let hasan_face = hasan_faces.first().expect("Couldn't get a Hasan face");

    let mut background = raster::open(background_file).unwrap();
    let mut face = raster::open(hasan_face).unwrap();
    let mut text = raster::open("text.png").unwrap();

    editor::resize(&mut background, 1280, 720, ResizeMode::Exact).unwrap();
    editor::resize(&mut face, 0, 720, ResizeMode::ExactHeight).unwrap();
    editor::resize(&mut text, 550, 0, ResizeMode::ExactWidth).unwrap();

    let background_with_face = editor::blend(
        &background,
        &face,
        BlendMode::Normal,
        1.0,
        PositionMode::BottomRight,
        0,
        0,
    )
    .unwrap();

    let thumbnail = editor::blend(
        &background_with_face,
        &text,
        BlendMode::Normal,
        1.0,
        PositionMode::TopLeft,
        50,
        50,
    )
    .unwrap();

    raster::save(&thumbnail, result_file).unwrap();
}
