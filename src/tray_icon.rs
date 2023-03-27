use std::{sync::mpsc, time::Duration};

use crate::utils::MpscSendExt;

#[derive(Debug)]
pub enum TrayInputEvent {
    Quit,
    RestartWork,
    SkipWork { by: Duration },
}

#[derive(Debug, Default)]
pub enum TrayItemMode {
    #[default]
    Normal,
    InBreak,
    Restricted,
}

pub struct TrayItem {
    tray_item: tray_item::TrayItem,
}

impl TrayItem {
    pub fn new_with_sender(
        mode: TrayItemMode,
        sender: &mpsc::SyncSender<TrayInputEvent>,
    ) -> Result<TrayItem, tray_item::TIError> {
        let mut tray_item = tray_item::TrayItem::new("Pomodoro SS", "timer_icon")?;
        match mode {
            TrayItemMode::Normal => {
                tray_item.add_label("Pomodoro SS")?;
                tray_item.add_label("Mode: Normal")?;

                tray_item.inner_mut().add_separator()?;

                let sender_cloned = sender.clone();
                tray_item.add_menu_item("Restart work", move || {
                    sender_cloned.just_send(TrayInputEvent::RestartWork)
                })?;

                let sender_cloned = sender.clone();
                tray_item.add_menu_item("Skip work 5 minutes", move || {
                    sender_cloned.just_send(TrayInputEvent::SkipWork {
                        by: Duration::from_secs(60 * 5),
                    })
                })?;

                let sender_cloned = sender.clone();
                tray_item.add_menu_item("Restart work with 10 minutes", move || {
                    sender_cloned.just_send(TrayInputEvent::SkipWork {
                        by: Duration::from_secs(60 * 10),
                    })
                })?;

                tray_item.inner_mut().add_separator()?;

                let sender_cloned = sender.clone();
                tray_item.add_menu_item("Quit", move || {
                    sender_cloned.just_send(TrayInputEvent::Quit)
                })?;
            }
            TrayItemMode::InBreak => {
                tray_item.add_label("Pomodoro SS")?;
                tray_item.add_label("Mode: In break")?;
            }
            TrayItemMode::Restricted => {
                tray_item.add_label("Pomodoro SS")?;
                tray_item.add_label("Mode: Restricted")?;
            }
        }
        Ok(TrayItem { tray_item })
    }

    pub fn switch_to(
        self,
        mode: TrayItemMode,
        sender: &mpsc::SyncSender<TrayInputEvent>,
    ) -> Result<TrayItem, tray_item::TIError> {
        drop(self);
        TrayItem::new_with_sender(mode, sender)
    }
}
