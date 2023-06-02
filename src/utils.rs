pub fn bytes_to_mib_gib(bytes: f32) -> String {
    if bytes > 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2}GiB", bytes / 1024.0 / 1024.0 / 1024.0)
    } else if bytes > 1024.0 * 1024.0 {
        format!("{:.2}MiB", bytes / 1024.0 / 1024.0)
    } else {
        format!("{:.2}KiB", bytes / 1024.0)
    }
}
