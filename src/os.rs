use anyhow::bail;

use crate::Result;

fn tray_item() -> Result<tray_item::TrayItem> {
    Ok(tray_item::TrayItem::new("Pomodoro SS", "timer_icon")?)
}

fn bool_to_cbool(b: bool) -> winapi::shared::minwindef::BOOL {
    if b {
        winapi::shared::minwindef::TRUE
    } else {
        winapi::shared::minwindef::FALSE
    }
}

fn block_input(block: bool) -> Result<()> {
    unsafe {
        let result = winapi::um::winuser::BlockInput(bool_to_cbool(block));
        if result == winapi::shared::minwindef::FALSE {
            bail!(std::io::Error::last_os_error())
        }
        Ok(())
    }
}

fn notify(summary: &str, body: &str) -> Result<()> {
    notify_rust::Notification::new()
        .summary(summary)
        .body(body)
        .show()?;
    Ok(())
}
