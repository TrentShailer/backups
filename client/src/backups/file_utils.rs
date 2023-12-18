use chrono::{DateTime, Local};

pub fn get_file_name() -> String {
    let datetime: DateTime<Local> = Local::now();
    datetime.format("%Y-%m-%d_%H-%M-%S.backup").to_string()
}
