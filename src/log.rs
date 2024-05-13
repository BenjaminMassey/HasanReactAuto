use std::io::Write;

const DIRECTORY: &str = "./logs/";

fn write(code: &str, message: &str) {
    let dtl = chrono::offset::Local::now();
    let date = dtl.format("%m-%d-%Y").to_string();
    let time = dtl.format("%H:%M:%S").to_string();
    let mut file = std::fs::File::options()
        .create(true)
        .write(true)
        .append(true)
        .open(format!("{DIRECTORY}{date}.txt"))
        .expect("Error opening log file.");
    let _ = file.write(&format!("{} | {} | {}\n", time, code, message).as_bytes());
    let _ = file.flush();
}

pub fn error(message: &str) {
    write("[ERROR]  ", message);
}

pub fn warning(message: &str) {
    write("[WARNING]", message);
}

pub fn info(message: &str) {
    write("[INFO]   ", message);
}