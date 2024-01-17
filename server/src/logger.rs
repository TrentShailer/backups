use owo_colors::{OwoColorize, Style};

pub fn init_fern() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            let message = message.to_string();

            let time = chrono::Local::now().format("%F %r").to_string();
            let level = record.level();
            let target = record.target();

            let style = match level {
                log::Level::Error => Style::new().red(),
                log::Level::Warn => Style::new().yellow(),
                log::Level::Info => Style::new().blue(),
                log::Level::Debug => Style::new().cyan(),
                log::Level::Trace => Style::new().cyan(),
            };

            let message = format_message(&message);

            out.finish(format_args!(
                "[{time}] [{level}] [{target}]\n{message}",
                time = time.style(style),
                level = level.style(style),
                target = target.style(style),
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
