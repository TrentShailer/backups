#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod ip_list;
mod logger;
mod server;
mod server_config;

use log::error;
use notify_rust::Notification;
use server::Server;

const BACKUP_PATH: &str = "./backups";

pub fn main() {
    logger::init_fern().unwrap();

    let mut server = match Server::new() {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to create server:\n{e}");
            if let Err(e) = Notification::new()
                .summary("Backups server failed to start.")
                .show()
            {
                error!("Failed to send notification:\n{e}");
            }
            return;
        }
    };

    loop {
        if let Err(e) = server.accept_blocking() {
            error!("Failed to accept and handle:\n{e}");

            let notification_message = match e {
                server::accept::Error::ReadHelloTls(_)
                | server::accept::Error::WriteAlert(_)
                | server::accept::Error::Accept(_)
                | server::accept::Error::AcceptTls(_) => {
                    "Backups server failed to accept connection."
                }

                _ => "Backups server failed to handle connection.",
            };

            if let Err(e) = Notification::new().summary(notification_message).show() {
                error!("Failed to send notification:\n{e}");
            }
        };
    }
}

#[cfg(test)]
mod tests;
