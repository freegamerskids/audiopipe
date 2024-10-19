#include <wchar.h>
#include <iostream>
#include <thread>

#include <wrl\implements.h>

#include <LoopbackCapture.h>

extern "C" void* loopback_capture_new()
{
    ComPtr<CLoopbackCapture> pLoopbackCapture = Make<CLoopbackCapture>();
    return pLoopbackCapture.Get();
}

extern "C" void loopback_capture_set_callback(void* loopback_capture_ptr, void* callback)
{
    ComPtr<CLoopbackCapture> pLoopbackCapture = Make<CLoopbackCapture>(loopback_capture_ptr);
    pLoopbackCapture->SetPacketCallback(callback);
}

extern "C" void loopback_capture_set_callback_user_data(void* loopback_capture_ptr, void* user_data)
{
    ComPtr<CLoopbackCapture> pLoopbackCapture = Make<CLoopbackCapture>(loopback_capture_ptr);
    pLoopbackCapture->SetPacketCallbackUserData(user_data);
}

extern "C" void loopback_capture_start(void* loopback_capture_ptr, const char* output_file_name, int process_id, bool include_process_tree)
{
    const size_t o_file_len = strlen(output_file_name)+1;
    wchar_t* wc = new wchar_t[o_file_len];
    mbstowcs (wc, output_file_name, o_file_len);

    ComPtr<CLoopbackCapture> pLoopbackCapture = Make<CLoopbackCapture>(loopback_capture_ptr);
    HRESULT hr = pLoopbackCapture->StartCaptureAsync(process_id, include_process_tree, wc);
    if (FAILED(hr))
    {
        wil::unique_hlocal_string message;
        FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_ALLOCATE_BUFFER, nullptr, hr,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), (PWSTR)&message, 0, nullptr);
        std::wcout << L"Failed to start capture\n0x" << std::hex << hr << L": " << message.get() << L"\n";
    }

    while (pLoopbackCapture->GetDeviceState() != DeviceState::Stopped)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
}

extern "C" void loopback_capture_stop(void* loopback_capture_ptr)
{
    ComPtr<CLoopbackCapture> pLoopbackCapture = Make<CLoopbackCapture>(loopback_capture_ptr);
    pLoopbackCapture->StopCaptureAsync();
}