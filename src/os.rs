use windows::core::Result as WindowsResult;

pub fn tray_item() -> Result<tray_item::TrayItem, tray_item::TIError> {
    tray_item::TrayItem::new("Pomodoro SS", "timer_icon")
}

pub fn block_input(block: bool) -> WindowsResult<()> {
    unsafe { windows::Win32::UI::Input::KeyboardAndMouse::BlockInput(block).ok() }
}

pub fn notify(summary: &str, body: &str) -> notify_rust::error::Result<()> {
    notify_rust::Notification::new()
        .appname("Pomodoro SS")
        .auto_icon()
        .summary(summary)
        .body(body)
        .show()?;
    Ok(())
}

#[test]
fn test_notify() {
    notify("hi", "hi");
}
