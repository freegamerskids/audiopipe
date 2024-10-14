
use std::{ffi::{c_char, c_void}, time::Duration};

use audiopipe::Application;
use wavers::{read, write, Samples};

pub fn main() -> iced::Result {

    /*iced::application("Iced Node Editor - Sockets Example", Application::update, Application::view)
        .theme(|_| iced::Theme::Dark)
        .antialiasing(true)
        .centered()
        .window_size(iced::Size { width: 800.0, height: 600.0 })
        .run()?;*/

    write("test_rust.wav", &[0i16], 48000, 2).unwrap();

    let mut stream = audiopipe::audio_capture::platform::windows::WindowsAudioCaptureStream::new(4652,Box::new(|packet| {
        let packet = packet.unwrap();

        let (samples, _sample_rate): (Samples<i16>,i32) = read::<i16, _>("test_rust.wav").unwrap();

        let samples: &[i16] = &samples.convert();
        let data = [samples, packet.data()].concat();
        write("test_rust.wav", data.as_slice(), 48000, 2).unwrap();
    })).unwrap();
    std::thread::sleep(Duration::from_millis(2000));
    stream.stop();

    /*let loopback_capture_ptr = unsafe { loopback_capture_start(b"testt.wav\0".as_ptr() as *const c_char, 4652, true) };

    std::thread::sleep(Duration::from_millis(10000));

    unsafe { loopback_capture_stop(loopback_capture_ptr) };*/

    Ok(())
}
