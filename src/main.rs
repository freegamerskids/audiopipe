
use std::time::Duration;

use audiopipe::Application;
use wavers::{read, write, Samples};

pub fn main() -> iced::Result {

    iced::application("Iced Node Editor - Sockets Example", Application::update, Application::view)
        .theme(|_| iced::Theme::Dark)
        .antialiasing(true)
        .centered()
        .window_size(iced::Size { width: 800.0, height: 600.0 })
        .run()?;

    let pid = 4652;
    println!("capturing audio for process with pid: {}", pid);

    write("test_rust.wav", &[0i16], 48000, 2).unwrap();

    let mut stream = audiopipe::platform::platform_impl::audio_capture::ImplAudioCaptureStream::new(pid,Box::new(|packet| {
        let packet = packet.unwrap();

        let (samples, _sample_rate): (Samples<i16>,i32) = read::<i16, _>("test_rust.wav").unwrap();

        let samples: &[i16] = &samples.convert();
        let data = [samples, packet.data()].concat();
        write("test_rust.wav", data.as_slice(), 48000, 2).unwrap();
    })).unwrap();
    std::thread::sleep(Duration::from_millis(2000));
    stream.stop();

    println!("capture stopped");

    Ok(())
}
