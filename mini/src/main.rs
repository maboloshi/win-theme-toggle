#![no_std]
#![no_main]
#![windows_subsystem = "windows"]

use core::panic::PanicInfo;

mod winapi {
    pub type HKEY = *mut core::ffi::c_void;
    pub const HKEY_CURRENT_USER: HKEY = 0x80000001 as *mut core::ffi::c_void;
    pub const KEY_READ: u32 = 0x20019;
    pub const KEY_WRITE: u32 = 0x20006;
    pub const REG_DWORD: u32 = 4;

    pub const HWND_BROADCAST: isize = 0xffff;
    pub const SMTO_ABORTIFHUNG: u32 = 0x0002;
    pub const WM_SETTINGCHANGE: u32 = 0x001A;
    pub const WM_THEMECHANGED: u32 = 0x031A;
    pub const WM_SYSCOLORCHANGE: u32 = 0x0015;

    #[link(name = "advapi32")]
    extern "system" {
        pub fn RegOpenKeyExW(
            hKey: HKEY,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut HKEY,
        ) -> i32;
        pub fn RegQueryValueExW(
            hKey: HKEY,
            lpValueName: *const u16,
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;
        pub fn RegSetValueExW(
            hKey: HKEY,
            lpValueName: *const u16,
            Reserved: u32,
            dwType: u32,
            lpData: *const u8,
            cbData: u32,
        ) -> i32;
        pub fn RegCloseKey(hKey: HKEY) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn SendMessageTimeoutW(
            hWnd: isize,
            Msg: u32,
            wParam: usize,
            lParam: isize,
            fuFlags: u32,
            uTimeout: u32,
            lpdwResult: *mut usize,
        ) -> isize;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn ExitProcess(exit_code: u32) -> !;
    }
}

// 极简的 UTF-16 转换
fn to_utf16(s: &str, buf: &mut [u16; 64]) {
    let bytes = s.as_bytes();
    let len = bytes.len().min(63);

    for i in 0..len {
        buf[i] = bytes[i] as u16;
    }
    buf[len] = 0;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { winapi::ExitProcess(1) }
}

#[no_mangle]
pub extern "system" fn mainCRTStartup() -> ! {
    unsafe {
        use core::ptr;

        // 使用栈上的缓冲区
        let mut path_buf = [0u16; 64];
        let mut apps_buf = [0u16; 64];
        let mut system_buf = [0u16; 64];
        let mut immersive_buf = [0u16; 64];

        to_utf16(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", &mut path_buf);
        to_utf16("AppsUseLightTheme", &mut apps_buf);
        to_utf16("SystemUsesLightTheme", &mut system_buf);
        to_utf16("ImmersiveColorSet", &mut immersive_buf);

        // 读取当前值
        let mut hkey: winapi::HKEY = ptr::null_mut();
        let mut current: u32 = 0;
        let mut size = 4;

        if winapi::RegOpenKeyExW(
            winapi::HKEY_CURRENT_USER,
            path_buf.as_ptr(),
            0,
            winapi::KEY_READ,
            &mut hkey,
        ) == 0
        {
            winapi::RegQueryValueExW(
                hkey,
                apps_buf.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut current as *mut _ as *mut u8,
                &mut size,
            );
            winapi::RegCloseKey(hkey);
        }

        // 切换值
        let new_value = if current == 0 { 1 } else { 0 };

        if winapi::RegOpenKeyExW(
            winapi::HKEY_CURRENT_USER,
            path_buf.as_ptr(),
            0,
            winapi::KEY_WRITE,
            &mut hkey,
        ) == 0
        {
            winapi::RegSetValueExW(
                hkey,
                apps_buf.as_ptr(),
                0,
                winapi::REG_DWORD,
                &new_value as *const _ as *const u8,
                4,
            );
            winapi::RegSetValueExW(
                hkey,
                system_buf.as_ptr(),
                0,
                winapi::REG_DWORD,
                &new_value as *const _ as *const u8,
                4,
            );
            winapi::RegCloseKey(hkey);
        }

        // 广播更改
        winapi::SendMessageTimeoutW(
            winapi::HWND_BROADCAST,
            winapi::WM_SETTINGCHANGE,
            0,
            immersive_buf.as_ptr() as isize,
            winapi::SMTO_ABORTIFHUNG,
            200,
            ptr::null_mut(),
        );
        
        // 目前实测发现 WM_THEMECHANGED 和 WM_SYSCOLORCHANGE 的广播非必须，暂时注释掉（调试用）
        // winapi::SendMessageTimeoutW(
        //     winapi::HWND_BROADCAST,
        //     winapi::WM_THEMECHANGED,
        //     0,
        //     0,
        //     winapi::SMTO_ABORTIFHUNG,
        //     200,
        //     ptr::null_mut(),
        // );
        // winapi::SendMessageTimeoutW(
        //     winapi::HWND_BROADCAST,
        //     winapi::WM_SYSCOLORCHANGE,
        //     0,
        //     0,
        //     winapi::SMTO_ABORTIFHUNG,
        //     200,
        //     ptr::null_mut(),
        // );

        winapi::ExitProcess(0);
    }
}