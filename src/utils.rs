use std::{ops::ControlFlow, sync::mpsc};

pub trait ResultIntoControLFlow<T, E> {
    fn break_err(self) -> ControlFlow<E, T>;
    fn break_res_err<TB>(self) -> ControlFlow<Result<TB, E>, T>;
}

impl<T, E> ResultIntoControLFlow<T, E> for Result<T, E> {
    fn break_err(self) -> ControlFlow<E, T> {
        match self {
            Ok(o) => ControlFlow::Continue(o),
            Err(e) => ControlFlow::Break(e),
        }
    }

    fn break_res_err<TB>(self) -> ControlFlow<Result<TB, E>, T> {
        match self {
            Ok(o) => ControlFlow::Continue(o),
            Err(e) => ControlFlow::Break(Err(e)),
        }
    }
}

pub trait MpscRecvExt<T> {
    fn maybe_recv(&self) -> Result<Option<T>, mpsc::TryRecvError>;
}

impl<T> MpscRecvExt<T> for mpsc::Receiver<T> {
    fn maybe_recv(&self) -> Result<Option<T>, mpsc::TryRecvError> {
        match self.try_recv() {
            Ok(o) => Ok(Some(o)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

pub trait MpscSendExt<T> {
    fn just_send(&self, v: T);
}

impl<T> MpscSendExt<T> for mpsc::SyncSender<T> {
    fn just_send(&self, v: T) {
        let _res = self.send(v);
    }
}
