use fern::colors::{Color, ColoredLevelConfig};

pub fn init_fern() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .trace(Color::BrightCyan)
        .debug(Color::BrightCyan)
        .info(Color::BrightBlue)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            let message = message.to_string();

            let cs = format!("\x1B[{}m", colors.get_color(&record.level()).to_fg_str());
            let ce = "\x1B[0m";

            let time = chrono::Local::now().format("%F %r").to_string();
            let level = record.level();
            let target = record.target();

            let message = format_message(&message);
            let message = message.replace("[cs]", &cs);
            let message = message.replace("[ce]", &ce);

            out.finish(format_args!(
                "[{cs}{time}{ce}] [{cs}{level}{ce}] [{cs}{target}{ce}]\n{message}",
                cs = cs,
                ce = ce,
                time = time,
                level = level,
                target = target,
                message = message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::DateBased::new("logs/", "%F.log.ansi"))
        .apply()?;

    Ok(())
}

pub fn format_message(message: &String) -> String {
    let parts = message.split("[br]");
    parts
        .enumerate()
        .flat_map(|(i, s)| {
            let parts = s.split("\n");

            parts.flat_map(move |s| {
                let output = format!("  {}{}\n", String::from("  ").repeat(i), s);
                output.as_str().chars().collect::<Vec<_>>()
            })
        })
        .collect::<String>()
}

pub fn format_message_short(message: &String) -> String {
    let parts = message.split("[br]");
    let mut parts_vec = parts.collect::<Vec<&str>>();
    parts_vec.pop();

    parts_vec
        .iter()
        .enumerate()
        .flat_map(|(i, s)| {
            let parts = s.split("\n");

            parts.flat_map(move |s| {
                let output = format!("  {}{}\n", String::from("  ").repeat(i), s);
                output.as_str().chars().collect::<Vec<_>>()
            })
        })
        .collect::<String>()
}
