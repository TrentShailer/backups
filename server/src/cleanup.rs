use crate::config::ProgramConfig;

pub fn spawn_cleanup_tasks(program_config: ProgramConfig) {
    for service in program_config.service_config {
        let service = service.clone();
        let backup_path = program_config.backup_path.clone();
        tokio::spawn(async move {
            // cleahup task
            for backup in service.backup_configs {
                backup.spawn_cleanup_task(backup_path.clone(), &service.folder_name);
            }
        });
    }
}
