use futures::task::{Context, Poll};
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::io::Read;
use std::path::Path;
use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::task::Waker;

pub const ASSETS_SCHEME: &str = "@assets://";

pub struct FileReadFuture {
    receiver: Receiver<Result<Vec<u8>, AndroidFileError>>,
    shared_waker: Arc<Mutex<Option<Waker>>>,
}

impl FileReadFuture {
    pub fn read_bytes_from_file<P: AsRef<Path>>(path: P) -> Self {
        let shared_waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        let (sender, receiver) = channel();
        let thread_shared_waker = shared_waker.clone();
        let filename = CString::new(path.as_ref().to_str().unwrap()).expect("CString::new failed");
        std::thread::spawn(move || {
            let mut shared_waker = thread_shared_waker.lock().unwrap();
            sender.send(read_bytes_from_file(&filename)).unwrap();
            if let Some(waker) = shared_waker.take() {
                waker.wake();
            }
        });

        FileReadFuture {
            receiver,
            shared_waker,
        }
    }
}

impl Future for FileReadFuture {
    type Output = Result<Vec<u8>, AndroidFileError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let mut shared_waker = self.shared_waker.lock().unwrap();
        if let Some(result) = self.receiver.try_recv().ok() {
            Poll::Ready(result)
        } else {
            *shared_waker = Some(ctx.waker().clone());
            Poll::Pending
        }
    }
}

pub fn create_writable_path() -> String {
    ndk_glue::native_activity()
        .internal_data_path()
        .to_str()
        .unwrap()
        .to_string()
}

fn read_bytes_from_file(filename: &CStr) -> Result<Vec<u8>, AndroidFileError> {
    let asset_manager = ndk_glue::native_activity().asset_manager();
    let mut asset = asset_manager
        .open(filename)
        .ok_or_else(|| AndroidFileError::OpenError)?;

    let mut bytes = vec![];
    asset
        .read_to_end(&mut bytes)
        .map_err(|_| AndroidFileError::ReadToEndError)?;
    Ok(bytes)
}

#[derive(Debug)]
pub enum AndroidFileError {
    OpenError,
    ReadToEndError,
}

impl Display for AndroidFileError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for AndroidFileError {}
