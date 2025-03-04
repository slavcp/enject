use windows::{
    core::*,
    Win32::{
        Foundation::BOOL,
        Foundation::*,
        System::{
            Diagnostics::{Debug::*, ToolHelp::*},
            LibraryLoader::*,
            Memory::*,
            Threading::*,
        },
        UI::WindowsAndMessaging::*,
    },
};

unsafe extern "system" fn find_window_by_pid(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = lparam.0 as *mut (HWND, u32);
    let target_pid = (*data).1;
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    if pid == target_pid {
        (*data).0 = hwnd;
        return BOOL(0);
    }
    BOOL(1)
}

unsafe fn get_proc(process_name: &str) -> Result<PROCESSENTRY32W> {
    let mut process_entry = PROCESSENTRY32W::default();
    process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();

    if Process32FirstW(snapshot, &mut process_entry).is_ok() {
        loop {
            let found_process_name = String::from_utf16_lossy(
                &process_entry.szExeFile[..process_entry
                    .szExeFile
                    .iter()
                    .position(|&x| x == 0)
                    .unwrap_or(process_entry.szExeFile.len())],
            );

            if found_process_name.eq(process_name) {
                let hwnd: HWND = Default::default();
                let pid = process_entry.th32ProcessID;
                let mut data = (hwnd, pid);

                let _ = EnumWindows(
                    Some(find_window_by_pid),
                    LPARAM(&mut data as *mut (HWND, u32) as _),
                );

                if data.0 != HWND::default() {
                    println!("Found window: {:?}", data.0);
                    return Ok(process_entry);
                }
            }

            if Process32NextW(snapshot, &mut process_entry).is_err() {
                break;
            }
        }
    }

    CloseHandle(snapshot).unwrap();
    Err(Error::new(
        E_FAIL,
        "error: unable to find process with window",
    ))
}

unsafe fn inject(process_id: u32) {
    println!("Targeting process ID: {}", process_id);
    let process_handle = OpenProcess(PROCESS_ALL_ACCESS, false, process_id).unwrap();
    let load_library = GetProcAddress(
        GetModuleHandleW(w!("kernel32.dll")).unwrap(),
        s!("LoadLibraryW"),
    )
    .unwrap();

    // ???????
    let dll_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("electron.dll");

    if !dll_path.exists() {
        println!("Error: DLL not found at path: {}", dll_path.display());
        return;
    }

    let dll_path = dll_path.to_str().unwrap().to_string();

    let dll_path_bytes: Vec<u16> = dll_path.encode_utf16().chain(std::iter::once(0)).collect();
    let alloc = VirtualAllocEx(
        process_handle,
        None,
        dll_path_bytes.len() * 2,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_READWRITE,
    );

    if let Err(e) = WriteProcessMemory(
        process_handle,
        alloc,
        dll_path_bytes.as_ptr() as _,
        dll_path_bytes.len() * 2,
        None,
    ) {
        println!("Failed to write process memory: {:?}", e);
        return;
    }

    if let Err(e) = CreateRemoteThread(
        process_handle,
        None,
        0,
        Some(std::mem::transmute(load_library)),
        Some(alloc),
        0,
        None,
    ) {
        println!("Failed to create remote thread: {:?}", e);
        return;
    }

    CloseHandle(process_handle).unwrap();
    println!("Injection successful!");
}

fn main() {
    let process_entry = loop {
        println!("Enter the process name:");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        let exe_name = input.trim().to_string();

        match unsafe { get_proc(&exe_name) } {
            Ok(entry) => break entry,
            Err(_) => {
                println!("Process '{}' not found.", exe_name);
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }
        }
    };

    unsafe { inject(process_entry.th32ProcessID) };

    println!("Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
}
