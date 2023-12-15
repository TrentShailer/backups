use crate::config::ProgramConfig;

pub fn valid_config(folder: &String, sub_folder: &String, program_config: &ProgramConfig) -> bool {
    let service = match program_config
        .service_config
        .iter()
        .find(|service| &service.folder_name == folder)
    {
        Some(v) => v,
        None => return false,
    };

    service
        .backup_configs
        .iter()
        .any(|backup| &backup.folder_name == sub_folder)
}
