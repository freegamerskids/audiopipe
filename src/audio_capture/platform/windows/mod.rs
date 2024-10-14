use std::{ffi::{c_char, c_void}, time::Duration};

extern "C" {
    fn loopback_capture_new() -> *mut c_void;
    fn loopback_capture_set_callback(loopback_capture_ptr: *mut c_void, callback: unsafe extern "C" fn(data: *const u8, num_frames: *const u32, flags: *const u32, device_position: *const u64, user_data: *mut c_void));
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

            unsafe extern "C" fn loopback_callback(data: *const u8, num_frames: *const u32, flags: *const u32, device_position: *const u64, user_data: *mut c_void) {
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

            loopback_capture_set_callback(loopback_capture_ptr, loopback_callback);

            loopback_capture_start(loopback_capture_ptr, b"test.wav\0".as_ptr() as *const c_char, process_id, true);

            Ok(WindowsAudioCaptureStream {
                loopback_capture_ptr
            })
        }

        /*unsafe {
            let should_couninit = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();

            let mm_device_enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| WindowsAudioCaptureStreamCreateError::Other(format!("Failed to create MMDeviceEnumerator: {}", e.to_string())))?;
            let device = mm_device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|_| WindowsAudioCaptureStreamCreateError::EndpointEnumerationFailed)?;
            
            let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)
                .map_err(|_| WindowsAudioCaptureStreamCreateError::AudioClientActivationFailed)?;

            let mut format = WAVEFORMATEX::default();
            format.wFormatTag = WAVE_FORMAT_PCM as u16;
            format.nSamplesPerSec = 48000;
            format.wBitsPerSample = 16;
            format.nChannels = 2;
            format.nBlockAlign = format.nChannels * 2;
            format.nAvgBytesPerSec = format.nSamplesPerSec * format.nBlockAlign as u32;
            format.cbSize = 0;

            let callback_format = format.clone();

            let buffer_size = 512;
            let buffer_time = buffer_size as i64 * 10000000i64 / format.nSamplesPerSec as i64;

            let buffer_duration = Duration::from_nanos(buffer_time as u64 * 100);
            let half_buffer_duration = buffer_duration / 2;

            audio_client.Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, buffer_time, buffer_time, &format as *const _, None)
                .map_err(|_| WindowsAudioCaptureStreamCreateError::AudioClientInitializeFailed)?;

            let capture_client : IAudioCaptureClient = audio_client.GetService()
                .map_err(|_| WindowsAudioCaptureStreamCreateError::AudioCaptureCreationFailed)?;

            let capture_client_send = SendCaptureClient::from_iaudiocaptureclient(capture_client);

            std::thread::spawn(move || {
                {
                    let should_couninit = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();

                    let mut last_device_position = 0u64;
                    let mut sample_count = 0u64;

                    let capture_client = capture_client_send.into_iaudiocaptureclient();
                    loop {
                        std::thread::sleep(half_buffer_duration);

                        let _buffered_count = match capture_client.GetNextPacketSize() {
                            Ok(count) => count,
                            Err(_) => {
                                (callback)(Err(WindowsAudioCaptureStreamError::Other(format!("Stream failed - couldn't fetch packet size"))));
                                break;
                            }
                        };

                        let mut data_ptr: *mut u8 = std::ptr::null_mut();

                        let mut num_frames = 0u32;
                        let mut flags = 0u32;
                        let mut device_position = 0u64;

                        match capture_client.GetBuffer(&mut data_ptr as *mut _, &mut num_frames as *mut _, &mut flags as *mut _, Some(&mut device_position as *mut _), None) {
                            Ok(_) => {
                                let packet = WindowsAudioCaptureStreamPacket {
                                    data: std::slice::from_raw_parts(data_ptr as *const i16, num_frames as usize * 2),
                                    channel_count: callback_format.nChannels as u32,
                                    origin_time: Duration::from_nanos(device_position as u64 * 100),
                                    duration: Duration::from_nanos((device_position - last_device_position) as u64),
                                    sample_index: sample_count
                                };
                                (callback)(Ok(packet));
                                let _ = capture_client.ReleaseBuffer(num_frames);
                                last_device_position = device_position;
                                sample_count += num_frames as u64;
                            },
                            Err(_) => {
                                (callback)(Err(WindowsAudioCaptureStreamError::GetBufferFailed));
                                break;
                            }
                        }

                    }

                    if should_couninit {
                        CoUninitialize();
                    }
                }
            });

            audio_client.Start()
                .map_err(|_| WindowsAudioCaptureStreamCreateError::StreamStartFailed)?;

            Ok(WindowsAudioCaptureStream {
                should_couninit,
                audio_client
            })
        }*/
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