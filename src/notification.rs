pub fn notify(summary: &str, body: &str) -> notify_rust::error::Result<()> {
    notify_rust::Notification::new()
        .appname("Pomodoro SS")
        .auto_icon()
        .summary(summary)
        .body(body)
        .show()?;
    Ok(())
}
