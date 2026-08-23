#![deny(unsafe_op_in_unsafe_fn)]

use std::collections::{HashMap, VecDeque};
use std::ffi::{c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(all(windows, feature = "ffmpeg"))]
mod encoder;
#[cfg(all(windows, feature = "ffmpeg"))]
mod finalizer;

const ABI_VERSION: u32 = 2;
const OK: i32 = 0;
const ERR_INVALID_ARGUMENT: i32 = -1;
const ERR_NOT_FOUND: i32 = -2;
const ERR_ALREADY_RUNNING: i32 = -3;
const ERR_NOT_RUNNING: i32 = -4;
const ERR_PLATFORM: i32 = -5;
const ERR_CAPTURE: i32 = -6;
const ERR_PANIC: i32 = -127;

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
static SESSIONS: OnceLock<Mutex<HashMap<u64, Arc<CaptureSession>>>> = OnceLock::new();
static LAST_ERROR: OnceLock<Mutex<String>> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CaptureConfig {
    pub window_title: String,
    pub fps: u32,
    pub queue_capacity: usize,
    pub show_cursor: bool,
    pub output_path: Option<String>,
    pub encoder: String,
    pub bitrate_kbps: u32,
    pub window_handle: u64,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            window_title: "Celeste".to_owned(),
            fps: 60,
            queue_capacity: 3,
            show_cursor: false,
            output_path: None,
            encoder: "auto".to_owned(),
            bitrate_kbps: 12_000,
            window_handle: 0,
        }
    }
}

impl CaptureConfig {
    fn validate(mut self) -> Result<Self, CaptureError> {
        self.window_title = self.window_title.trim().to_owned();
        if self.window_title.is_empty() {
            return Err(CaptureError::InvalidConfig("window_title is empty"));
        }
        if !(1..=240).contains(&self.fps) {
            return Err(CaptureError::InvalidConfig("fps must be between 1 and 240"));
        }
        if !(1..=16).contains(&self.queue_capacity) {
            return Err(CaptureError::InvalidConfig(
                "queue_capacity must be between 1 and 16",
            ));
        }
        if let Some(path) = self.output_path.as_mut() {
            *path = path.trim().to_owned();
            if path.is_empty() {
                return Err(CaptureError::InvalidConfig("output_path is empty"));
            }
        }
        self.encoder = self.encoder.trim().to_owned();
        if self.encoder.is_empty() {
            self.encoder = "auto".to_owned();
        }
        if !(100..=200_000).contains(&self.bitrate_kbps) {
            return Err(CaptureError::InvalidConfig(
                "bitrate_kbps must be between 100 and 200000",
            ));
        }
        Ok(self)
    }
}

#[derive(Debug, Error)]
enum CaptureError {
    #[error("invalid capture config: {0}")]
    InvalidConfig(&'static str),
    #[error("capture window not found: {0}")]
    WindowNotFound(String),
    #[error("scap capture is unavailable: {0}")]
    Scap(String),
    #[error("screen capture is only available on Windows in this build")]
    UnsupportedPlatform,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CaptureStats {
    pub abi_version: u32,
    pub running: u32,
    pub width: u32,
    pub height: u32,
    pub queue_depth: u32,
    pub frames_captured: u64,
    pub frames_consumed: u64,
    pub frames_dropped: u64,
    pub bytes_captured: u64,
    pub last_frame_unix_nanos: u64,
    pub media_time_nanos: u64,
}

#[derive(Debug)]
struct CapturedFrame {
    width: u32,
    height: u32,
    captured_at_unix_nanos: u64,
    bgra: Vec<u8>,
}

#[derive(Debug)]
struct QueueState {
    frames: VecDeque<CapturedFrame>,
    closed: bool,
}

#[derive(Debug)]
struct LatestFrameQueue {
    capacity: usize,
    state: Mutex<QueueState>,
    available: Condvar,
}

impl LatestFrameQueue {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            state: Mutex::new(QueueState {
                frames: VecDeque::with_capacity(capacity),
                closed: false,
            }),
            available: Condvar::new(),
        }
    }

    fn push_latest(&self, frame: CapturedFrame) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.closed {
            return true;
        }
        let dropped = if state.frames.len() == self.capacity {
            state.frames.pop_front();
            true
        } else {
            false
        };
        state.frames.push_back(frame);
        self.available.notify_one();
        dropped
    }

    fn pop(&self) -> Option<CapturedFrame> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        loop {
            if let Some(frame) = state.frames.pop_front() {
                return Some(frame);
            }
            if state.closed {
                return None;
            }
            state = self
                .available
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn close(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.closed = true;
        self.available.notify_all();
    }

    fn depth(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .frames
            .len()
    }
}

#[derive(Debug, Default)]
struct AtomicStats {
    width: AtomicU32,
    height: AtomicU32,
    frames_captured: AtomicU64,
    frames_consumed: AtomicU64,
    frames_dropped: AtomicU64,
    bytes_captured: AtomicU64,
    last_frame_unix_nanos: AtomicU64,
    media_time_nanos: AtomicU64,
}

#[derive(Debug)]
struct CaptureSession {
    config: CaptureConfig,
    running: AtomicBool,
    stop_requested: AtomicBool,
    capture_finished: AtomicBool,
    queue: Arc<LatestFrameQueue>,
    stats: Arc<AtomicStats>,
    capture_thread: Mutex<Option<JoinHandle<()>>>,
    consumer_thread: Mutex<Option<JoinHandle<()>>>,
}

impl CaptureSession {
    fn new(config: CaptureConfig) -> Self {
        Self {
            queue: Arc::new(LatestFrameQueue::new(config.queue_capacity)),
            config,
            running: AtomicBool::new(false),
            stop_requested: AtomicBool::new(false),
            capture_finished: AtomicBool::new(true),
            stats: Arc::new(AtomicStats::default()),
            capture_thread: Mutex::new(None),
            consumer_thread: Mutex::new(None),
        }
    }

    fn start(self: &Arc<Self>) -> Result<(), i32> {
        if self.running.swap(true, Ordering::AcqRel) {
            return Err(ERR_ALREADY_RUNNING);
        }
        self.stop_requested.store(false, Ordering::Release);
        self.capture_finished.store(false, Ordering::Release);

        let capture_session = Arc::clone(self);
        let capture = thread::Builder::new()
            .name("microblocks-qol-wgc".to_owned())
            .spawn(move || {
                match catch_unwind(AssertUnwindSafe(|| run_capture(&capture_session))) {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => set_last_error(error.to_string()),
                    Err(payload) => set_last_error(format!(
                        "scap capture thread panicked: {}",
                        panic_payload_message(payload)
                    )),
                }
                capture_session
                    .capture_finished
                    .store(true, Ordering::Release);
                capture_session.queue.close();
                capture_session.running.store(false, Ordering::Release);
            })
            .map_err(|error| {
                self.running.store(false, Ordering::Release);
                set_last_error(format!("failed to spawn capture thread: {error}"));
                ERR_CAPTURE
            })?;
        *self
            .capture_thread
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(capture);

        let consumer_session = Arc::clone(self);
        let consumer = thread::Builder::new()
            .name("microblocks-qol-encoder-feed".to_owned())
            .spawn(move || {
                match catch_unwind(AssertUnwindSafe(|| run_consumer(&consumer_session))) {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => {
                        set_last_error(error);
                        consumer_session
                            .stop_requested
                            .store(true, Ordering::Release);
                    }
                    Err(payload) => {
                        set_last_error(format!(
                            "FFmpeg consumer thread panicked: {}",
                            panic_payload_message(payload)
                        ));
                        consumer_session
                            .stop_requested
                            .store(true, Ordering::Release);
                    }
                }
            })
            .map_err(|error| {
                self.stop_requested.store(true, Ordering::Release);
                set_last_error(format!("failed to spawn encoder-feed thread: {error}"));
                ERR_CAPTURE
            })?;
        *self
            .consumer_thread
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(consumer);
        Ok(())
    }

    fn stop(&self) -> Result<(), i32> {
        if !self.running.load(Ordering::Acquire) && self.capture_finished.load(Ordering::Acquire) {
            self.join_finished_threads();
            return Err(ERR_NOT_RUNNING);
        }
        self.stop_requested.store(true, Ordering::Release);

        // A WGC window can stop producing frames while minimized. Do not block the game thread
        // indefinitely: join only after the capture callback has observed the stop request.
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !self.capture_finished.load(Ordering::Acquire) && std::time::Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(5));
        }
        if self.capture_finished.load(Ordering::Acquire) {
            self.join_finished_threads();
        }
        Ok(())
    }

    fn join_finished_threads(&self) {
        if let Some(thread) = self
            .capture_thread
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = thread.join();
        }
        self.queue.close();
        if let Some(thread) = self
            .consumer_thread
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = thread.join();
        }
    }

    fn stats(&self) -> CaptureStats {
        CaptureStats {
            abi_version: ABI_VERSION,
            running: u32::from(self.running.load(Ordering::Acquire)),
            width: self.stats.width.load(Ordering::Relaxed),
            height: self.stats.height.load(Ordering::Relaxed),
            queue_depth: self.queue.depth().try_into().unwrap_or(u32::MAX),
            frames_captured: self.stats.frames_captured.load(Ordering::Relaxed),
            frames_consumed: self.stats.frames_consumed.load(Ordering::Relaxed),
            frames_dropped: self.stats.frames_dropped.load(Ordering::Relaxed),
            bytes_captured: self.stats.bytes_captured.load(Ordering::Relaxed),
            last_frame_unix_nanos: self.stats.last_frame_unix_nanos.load(Ordering::Relaxed),
            media_time_nanos: self.stats.media_time_nanos.load(Ordering::Relaxed),
        }
    }
}

fn run_consumer(session: &Arc<CaptureSession>) -> Result<(), String> {
    #[cfg(all(windows, not(feature = "ffmpeg")))]
    if session.config.output_path.is_some() {
        return Err("this native library was built without FFmpeg encoding support".to_owned());
    }
    #[cfg(all(windows, feature = "ffmpeg"))]
    let mut encoder = None;
    while let Some(frame) = session.queue.pop() {
        #[cfg(all(windows, feature = "ffmpeg"))]
        if session.config.output_path.is_some() {
            if encoder.is_none() {
                encoder = Some(
                    encoder::VideoFileEncoder::create(&session.config, &frame)
                        .map_err(|error| error.to_string())?,
                );
            }
            encoder
                .as_mut()
                .expect("encoder initialized above")
                .encode(&frame)
                .map_err(|error| error.to_string())?;
        }
        session
            .stats
            .frames_consumed
            .fetch_add(1, Ordering::Relaxed);
    }
    #[cfg(all(windows, feature = "ffmpeg"))]
    if let Some(mut encoder) = encoder {
        encoder.finish().map_err(|error| error.to_string())?;
    }
    Ok(())
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        self.stop_requested.store(true, Ordering::Release);
        self.queue.close();
    }
}

#[cfg(windows)]
fn run_capture(session: &Arc<CaptureSession>) -> Result<(), CaptureError> {
    use scap::capturer::{Capturer, Options, Resolution};
    use scap::frame::{Frame, FrameType, VideoFrame};

    let target = find_capture_window(&session.config)
        .ok_or_else(|| CaptureError::WindowNotFound(session.config.window_title.clone()))?;

    let mut capturer = Capturer::build(Options {
        fps: session.config.fps,
        show_cursor: session.config.show_cursor,
        show_highlight: false,
        target: Some(scap::Target::Window(target)),
        crop_area: None,
        output_type: FrameType::BGRAFrame,
        output_resolution: Resolution::Captured,
        excluded_targets: None,
        captures_audio: false,
        exclude_current_process_audio: false,
    })
    .map_err(|error| CaptureError::Scap(error.to_string()))?;

    capturer.start_capture();
    let mut origin_unix_nanos = None;
    while !session.stop_requested.load(Ordering::Acquire) {
        let Some(frame) = capturer
            .get_next_frame_timeout(Duration::from_millis(100))
            .map_err(|error| CaptureError::Scap(error.to_string()))?
        else {
            continue;
        };
        let Frame::Video(VideoFrame::BGRA(frame)) = frame else {
            continue;
        };
        let captured_at_unix_nanos = frame
            .display_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX);
        let origin = *origin_unix_nanos.get_or_insert(captured_at_unix_nanos);
        let media_time_nanos = captured_at_unix_nanos.saturating_sub(origin);
        let captured = CapturedFrame {
            width: frame.width.max(0) as u32,
            height: frame.height.max(0) as u32,
            captured_at_unix_nanos,
            bgra: frame.data,
        };
        session.stats.width.store(captured.width, Ordering::Relaxed);
        session
            .stats
            .height
            .store(captured.height, Ordering::Relaxed);
        session
            .stats
            .frames_captured
            .fetch_add(1, Ordering::Relaxed);
        session
            .stats
            .bytes_captured
            .fetch_add(captured.bgra.len() as u64, Ordering::Relaxed);
        session
            .stats
            .last_frame_unix_nanos
            .store(captured_at_unix_nanos, Ordering::Relaxed);
        session
            .stats
            .media_time_nanos
            .store(media_time_nanos, Ordering::Relaxed);
        if session.queue.push_latest(captured) {
            session.stats.frames_dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    capturer.stop_capture();
    Ok(())
}

#[cfg(windows)]
fn find_capture_window(config: &CaptureConfig) -> Option<scap::Window> {
    use std::ffi::c_void;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;

    if config.window_handle != 0 {
        let raw_handle = HWND(config.window_handle as usize as *mut c_void);
        if unsafe { IsWindow(raw_handle).as_bool() } {
            return Some(scap::Window {
                id: config.window_handle as u32,
                title: config.window_title.clone(),
                raw_handle,
            });
        }
    }
    find_current_process_window(&config.window_title).or_else(|| {
        scap::get_all_targets()
            .into_iter()
            .filter_map(|target| match target {
                scap::Target::Window(window) => Some(window),
                scap::Target::Display(_) => None,
            })
            .find(|window| {
                window.title == config.window_title || window.title.contains(&config.window_title)
            })
    })
}

#[cfg(windows)]
fn find_current_process_window(title: &str) -> Option<scap::Window> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE};
    use windows::Win32::System::Threading::GetCurrentProcessId;
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClientRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible,
    };

    struct Search {
        process_id: u32,
        title: String,
        found: Option<(HWND, String)>,
        fallback: Option<(HWND, String, u64)>,
    }

    unsafe extern "system" fn visit(window: HWND, parameter: LPARAM) -> BOOL {
        // SAFETY: `parameter` points to the Search value for the duration of EnumWindows.
        let search = unsafe { &mut *(parameter.0 as *mut Search) };
        let mut process_id = 0;
        unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
        if process_id != search.process_id {
            return TRUE;
        }
        let length = unsafe { GetWindowTextLengthW(window) }.max(0);
        let mut utf16 = vec![0_u16; length as usize + 1];
        let copied = unsafe { GetWindowTextW(window, &mut utf16) }.max(0);
        let actual = String::from_utf16_lossy(&utf16[..copied as usize]);
        let mut rect = RECT::default();
        let _ = unsafe { GetClientRect(window, &mut rect) };
        let area = (rect.right - rect.left).max(0) as u64 * (rect.bottom - rect.top).max(0) as u64;
        let visible_bonus = u64::from(unsafe { IsWindowVisible(window).as_bool() }) << 63;
        let score = visible_bonus | area.min(i64::MAX as u64);
        if search
            .fallback
            .as_ref()
            .is_none_or(|(_, _, previous_score)| score > *previous_score)
        {
            search.fallback = Some((window, actual.clone(), score));
        }
        if !actual.is_empty() && (actual == search.title || actual.contains(&search.title)) {
            search.found = Some((window, actual));
            return BOOL(0);
        }
        TRUE
    }

    let mut search = Search {
        process_id: unsafe { GetCurrentProcessId() },
        title: title.to_owned(),
        found: None,
        fallback: None,
    };
    let _ = unsafe { EnumWindows(Some(visit), LPARAM(std::ptr::addr_of_mut!(search) as isize)) };
    search
        .found
        .or_else(|| search.fallback.map(|(window, title, _)| (window, title)))
        .map(|(raw_handle, actual_title)| scap::Window {
            id: raw_handle.0 as usize as u32,
            title: actual_title,
            raw_handle,
        })
}

#[cfg(not(windows))]
fn run_capture(_session: &Arc<CaptureSession>) -> Result<(), CaptureError> {
    Err(CaptureError::UnsupportedPlatform)
}

fn sessions() -> &'static Mutex<HashMap<u64, Arc<CaptureSession>>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn set_last_error(message: impl Into<String>) {
    *LAST_ERROR
        .get_or_init(|| Mutex::new(String::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = message.into();
}

fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}

fn ffi_status(operation: impl FnOnce() -> Result<i32, i32>) -> i32 {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(status)) => status,
        Ok(Err(status)) => status,
        Err(_) => {
            set_last_error("native capture panicked");
            ERR_PANIC
        }
    }
}

unsafe fn utf8_from_raw<'a>(pointer: *const u8, length: usize) -> Result<&'a str, i32> {
    if pointer.is_null() || length == 0 {
        set_last_error("null or empty UTF-8 input");
        return Err(ERR_INVALID_ARGUMENT);
    }
    // SAFETY: The caller promises that `pointer..pointer+length` is readable for this call.
    let bytes = unsafe { std::slice::from_raw_parts(pointer, length) };
    std::str::from_utf8(bytes).map_err(|error| {
        set_last_error(format!("invalid UTF-8 input: {error}"));
        ERR_INVALID_ARGUMENT
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mqol_capture_abi_version() -> u32 {
    ABI_VERSION
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mqol_capture_create(
    config_json: *const u8,
    config_length: usize,
    output_handle: *mut u64,
) -> i32 {
    ffi_status(|| {
        if output_handle.is_null() {
            set_last_error("output_handle is null");
            return Err(ERR_INVALID_ARGUMENT);
        }
        // SAFETY: Validated above and required by the exported ABI contract.
        let json = unsafe { utf8_from_raw(config_json, config_length)? };
        let config = serde_json::from_str::<CaptureConfig>(json)
            .map_err(|error| {
                set_last_error(format!("invalid capture config JSON: {error}"));
                ERR_INVALID_ARGUMENT
            })?
            .validate()
            .map_err(|error| {
                set_last_error(error.to_string());
                ERR_INVALID_ARGUMENT
            })?;
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        sessions()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(handle, Arc::new(CaptureSession::new(config)));
        // SAFETY: `output_handle` was checked for null and points to caller-owned writable memory.
        unsafe { ptr::write(output_handle, handle) };
        Ok(OK)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mqol_capture_start(handle: u64) -> i32 {
    ffi_status(|| {
        let session = sessions()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or(ERR_NOT_FOUND)?;
        session.start()?;
        Ok(OK)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mqol_capture_stop(handle: u64) -> i32 {
    ffi_status(|| {
        let session = sessions()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or(ERR_NOT_FOUND)?;
        session.stop()?;
        Ok(OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mqol_capture_get_stats(handle: u64, output: *mut CaptureStats) -> i32 {
    ffi_status(|| {
        if output.is_null() {
            set_last_error("stats output pointer is null");
            return Err(ERR_INVALID_ARGUMENT);
        }
        let session = sessions()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or(ERR_NOT_FOUND)?;
        // SAFETY: `output` was checked for null and is writable for one CaptureStats value.
        unsafe { ptr::write(output, session.stats()) };
        Ok(OK)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mqol_capture_destroy(handle: u64) -> i32 {
    ffi_status(|| {
        let session = sessions()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&handle)
            .ok_or(ERR_NOT_FOUND)?;
        let _ = session.stop();
        Ok(OK)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mqol_capture_last_error(buffer: *mut c_char, capacity: usize) -> usize {
    let message = LAST_ERROR
        .get_or_init(|| Mutex::new(String::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let required = message.len() + 1;
    if buffer.is_null() || capacity == 0 {
        return required;
    }
    let copied = message.len().min(capacity.saturating_sub(1));
    // SAFETY: The caller provides `capacity` writable bytes. We copy at most capacity - 1 and
    // always append a trailing NUL.
    unsafe {
        ptr::copy_nonoverlapping(message.as_ptr(), buffer.cast::<u8>(), copied);
        ptr::write(buffer.add(copied), 0);
    }
    required
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mqol_recording_finalize(plan_json: *const u8, plan_length: usize) -> i32 {
    ffi_status(|| {
        #[cfg(all(windows, feature = "ffmpeg"))]
        {
            // SAFETY: The exported ABI requires a readable UTF-8 buffer for this call.
            let json = unsafe { utf8_from_raw(plan_json, plan_length)? };
            let plan = serde_json::from_str::<finalizer::FinalizePlan>(json).map_err(|error| {
                set_last_error(format!("invalid finalize plan JSON: {error}"));
                ERR_INVALID_ARGUMENT
            })?;
            finalizer::finalize(&plan).map_err(|error| {
                set_last_error(error.to_string());
                ERR_CAPTURE
            })?;
            Ok(OK)
        }
        #[cfg(not(all(windows, feature = "ffmpeg")))]
        {
            let _ = (plan_json, plan_length);
            set_last_error("native FFmpeg finalization is unavailable in this build");
            Err(ERR_PLATFORM)
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mqol_capture_reserved(_value: *mut c_void) -> i32 {
    ERR_PLATFORM
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(id: u8) -> CapturedFrame {
        CapturedFrame {
            width: 1,
            height: 1,
            captured_at_unix_nanos: id as u64,
            bgra: vec![id; 4],
        }
    }

    #[test]
    fn bounded_queue_drops_oldest_frame() {
        let queue = LatestFrameQueue::new(2);
        assert!(!queue.push_latest(frame(1)));
        assert!(!queue.push_latest(frame(2)));
        assert!(queue.push_latest(frame(3)));
        assert_eq!(queue.pop().unwrap().captured_at_unix_nanos, 2);
        assert_eq!(queue.pop().unwrap().captured_at_unix_nanos, 3);
    }

    #[test]
    fn config_rejects_unbounded_memory_settings() {
        assert!(
            CaptureConfig {
                queue_capacity: 0,
                ..CaptureConfig::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            CaptureConfig {
                queue_capacity: 17,
                ..CaptureConfig::default()
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn ffi_create_and_destroy_roundtrip() {
        let json = br#"{"window_title":"Celeste","fps":60,"queue_capacity":3}"#;
        let mut handle = 0;
        // SAFETY: The test provides valid pointers and lengths.
        assert_eq!(
            unsafe { mqol_capture_create(json.as_ptr(), json.len(), &mut handle) },
            OK
        );
        assert_ne!(handle, 0);
        let mut stats = CaptureStats::default();
        // SAFETY: The output pointer is valid for one CaptureStats.
        assert_eq!(unsafe { mqol_capture_get_stats(handle, &mut stats) }, OK);
        assert_eq!(stats.abi_version, ABI_VERSION);
        assert_eq!(mqol_capture_destroy(handle), OK);
    }
}
