use std::collections::HashMap;

use windows::Win32::{
    Foundation::{BOOL, HMODULE, HWND, LPARAM, MAX_PATH, RECT}, 
    System::{ProcessStatus::GetModuleFileNameExW, Threading::{GetCurrentProcessId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION}}, 
    UI::WindowsAndMessaging::{EnumWindows, GetWindowRect, GetWindowThreadProcessId, IsWindowVisible}
};

pub fn get_processes_with_windows() -> HashMap<u32, String> {
    let mut processes: HashMap<u32, String> = HashMap::new();

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if !IsWindowVisible(hwnd).as_bool() { return true.into() }

        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).unwrap();
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if width == 0 || height == 0 { return true.into() }
        
        let processes = &mut *(lparam.0 as *mut HashMap<u32, String>);

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let current_process_id = GetCurrentProcessId();
        if process_id == current_process_id { return true.into() }

        if processes.contains_key(&process_id) { return true.into() }

        let h_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id);
        if h_process.is_err() {
            return true.into();
        }

        let mut process_name: String = "".to_string();

        let mut module_name = vec![0u16; MAX_PATH as usize];
        let size = GetModuleFileNameExW(h_process.unwrap(), HMODULE(0 as _), &mut module_name);
        if size > 0 {
            module_name.truncate(size as usize);
            if let Ok(name) = String::from_utf16(&module_name) {
                process_name = name.split(&"\\".to_string()).last().unwrap().to_string();
            }
        }

        if processes.values().find(|s| **s == process_name).is_some() { return true.into() }

        processes.insert(process_id, process_name);

        true.into()
    }

    unsafe {
        EnumWindows(Some(enum_windows_proc), LPARAM(&mut processes as *mut _ as isize)).unwrap();
    }

    processes
}