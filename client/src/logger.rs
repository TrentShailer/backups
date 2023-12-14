/*
    This code is completely fucked because I have no idea what I am doing with macros
    and I don't think they are meant to be used this way.
    However, this is solution works as long as I give it the intended data.
    It keeps the code for where it is called clean and it will only be used by me.
*/

#[macro_export]
macro_rules! backup_error {
    ($service:expr, $backup:expr, $($args:expr),+ ) => {
		let error = format!("{}", format_args!($($args),+));
        let parts = error.split("\n");
    let error_string = parts
        .enumerate()
        .flat_map(|(i, s)| {
            let output = format!("      {}{}\n", String::from("  ").repeat(i), s);
            output.as_str().chars().collect::<Vec<_>>()
        })
        .collect::<String>();

        log::error!(
            "\n  {}/{}\n    BackupError\n{}",
            $service,
            $backup,
            error_string
        );
    };
}

#[macro_export]
macro_rules! app_error {
    ($($args:expr),+ ) => {
		let error = format!("{}", format_args!($($args),+));
        let parts = error.split("\n");
    let error_string = parts
        .enumerate()
        .flat_map(|(i, s)| {
            let output = format!("      {}{}\n", String::from("  ").repeat(i), s);
            output.as_str().chars().collect::<Vec<_>>()
        })
        .collect::<String>();

        log::error!(
            "\n  AppError\n{}",
            error_string
        );
    };
}
