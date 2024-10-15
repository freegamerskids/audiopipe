use std::{ffi::{c_char, c_void}, time::Duration};

extern "C" {
    fn loopback_capture_new() -> *mut c_void;
    fn loopback_capture_set_callback(loopback_capture_ptr: *mut c_void, callback: *mut c_void); //unsafe extern "C" fn(*const u8, *const u32, *const u32, *const u64, *mut c_void)
    fn loopback_capture_set_callback_user_data(loopback_capture_ptr: *mut c_void, user_data: *mut c_void);
    fn loopback_capture_start(loopback_capture_ptr: *mut c_void, output_file_name: *const c_char, process_id: i32, include_process_tree: bool) -> *mut c_void;
    fn loopback_capture_stop(loopback_capture_ptr: *mut c_void);
}

pub struct WindowsAudioCaptureStream {
    loopback_capture_ptr: *mut c_void
}

#[derive(Debug)]
pub enum WindowsAudioCaptureStreamCreateError {
    Other(String),
    EndpointEnumerationFailed,
    AudioClientActivationFailed,
    AudioClientInitializeFailed,
    AudioCaptureCreationFailed,
    StreamStartFailed,
}

#[derive(Debug)]
pub enum WindowsAudioCaptureStreamError {
    Other(String),
    GetBufferFailed,
}

#[allow(unused)]
pub struct WindowsAudioCaptureStreamPacket<'a> {
    pub(crate) data: &'a [i16],
    pub(crate) channel_count: u32,
    pub(crate) origin_time: Duration,
    pub(crate) duration: Duration,
    pub(crate) sample_index: u64,
}

impl WindowsAudioCaptureStreamPacket<'_> {
    pub fn data(&self) -> &[i16] {
        self.data
    }
}

struct LoopbackCapture(*mut c_void);

unsafe impl std::marker::Send for LoopbackCapture {}
unsafe impl std::marker::Sync for LoopbackCapture {}

impl LoopbackCapture {
    fn from_loopback_capture_ptr(loopback_capture_ptr: *mut c_void) -> Self {
        LoopbackCapture(loopback_capture_ptr)
    }

    fn to_ptr(&self) -> *mut c_void {
        self.0
    }
}

struct CCallbackUserData {
    loopback_capture_ptr: *mut c_void,
    callback: Box<dyn for <'a> FnMut(Result<WindowsAudioCaptureStreamPacket<'a>, WindowsAudioCaptureStreamError>) + Send + 'static>,
    last_device_position: u64,
    sample_count: u64,
}

impl WindowsAudioCaptureStream {
    pub fn new(process_id:i32, callback: Box<dyn for <'a> FnMut(Result<WindowsAudioCaptureStreamPacket<'a>, WindowsAudioCaptureStreamError>) + Send + 'static>) -> Result<Self, WindowsAudioCaptureStreamCreateError> {
        unsafe {
            let loopback_capture_ptr = loopback_capture_new();

            let user_data = CCallbackUserData {
                loopback_capture_ptr,
                callback,
                last_device_position: 0,
                sample_count: 0
            };

            loopback_capture_set_callback_user_data(loopback_capture_ptr, Box::into_raw(Box::new(user_data)) as *mut c_void);

            unsafe extern "C" fn loopback_callback(data: *mut u8, num_frames: *const u32, flags: *const u32, device_position: *const u64, user_data: *mut c_void) {
                let callback_user_data = &mut *(user_data as *mut CCallbackUserData);

                let data = std::slice::from_raw_parts(data as *const i16, *num_frames as usize * 2);
                let _flags = *flags;
                let device_position = *device_position;
                let packet = WindowsAudioCaptureStreamPacket {
                    data,
                    channel_count: 2,
                    origin_time: Duration::from_nanos(device_position as u64 * 100),
                    duration: Duration::from_nanos((device_position - callback_user_data.last_device_position) as u64),
                    sample_index: callback_user_data.sample_count
                };
                (callback_user_data.callback)(Ok(packet));
                callback_user_data.last_device_position = device_position;
                callback_user_data.sample_count += num_frames as u64;

                loopback_capture_set_callback_user_data(callback_user_data.loopback_capture_ptr, user_data as *mut c_void);
            }

            loopback_capture_set_callback(loopback_capture_ptr, loopback_callback as *mut c_void);

            let send_loopback_capture = LoopbackCapture::from_loopback_capture_ptr(loopback_capture_ptr);

            std::thread::spawn(move || {
                loopback_capture_start(send_loopback_capture.to_ptr(), b"test.wav\0".as_ptr() as *const c_char, process_id, true);
            });

            Ok(WindowsAudioCaptureStream {
                loopback_capture_ptr
            })
        }
    }

    pub fn stop(&mut self) {
        unsafe {
            loopback_capture_stop(self.loopback_capture_ptr);
        }
    }
}

impl Drop for WindowsAudioCaptureStream {
    fn drop(&mut self) {
        unsafe {
            loopback_capture_stop(self.loopback_capture_ptr);
        }
    }
}