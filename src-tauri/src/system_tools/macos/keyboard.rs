use crate::error::AppError;

pub fn paste() -> Result<(), AppError> {
    std::process::Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events" to keystroke "v" using command down"#,
        ])
        .output()?;
    Ok(())
}
