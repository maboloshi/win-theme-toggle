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
    pub const REG_SZ: u32 = 1;

    pub const HWND_BROADCAST: isize = 0xffff;
    pub const WM_SETTINGCHANGE: u32 = 0x001A;
    pub const WM_THEMECHANGED: u32 = 0x031A;
    pub const WM_DWMCOLORIZATIONCOLORCHANGED: u32 = 0x0320;

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
        pub fn SendNotifyMessageW(
            hWnd: isize,
            Msg: u32,
            wParam: usize,
            lParam: isize,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn ExitProcess(exit_code: u32) -> !;
    }
}

// 将 &str 转换为栈上的 UTF-16 空结尾字符串（最多 63 字符）
macro_rules! wstr {
    ($s:literal) => {{
        let bytes = $s.as_bytes();
        let len = bytes.len().min(63);
        let mut buf = [0u16; 64];
        let mut i = 0;
        while i < len {
            buf[i] = bytes[i] as u16;
            i += 1;
        }
        buf
    }};
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { winapi::ExitProcess(1) }
}

// 异步广播：SendNotifyMessageW 对于其他线程的窗口 = PostMessageW（纯异步）
// 投递消息到所有顶层窗口的消息队列后立即返回，总耗时 <1ms
macro_rules! notify_all {
    ($msg:expr, $lparam:expr) => {
        winapi::SendNotifyMessageW(
            winapi::HWND_BROADCAST, $msg, 0, $lparam,
        );
    };
}

#[no_mangle]
pub extern "system" fn mainCRTStartup() -> ! {
    unsafe {
        use core::ptr;

        // === 读取、反转当前主题值（基于 SystemUsesLightTheme） ===
        let mut hkey: winapi::HKEY = ptr::null_mut();
        if winapi::RegOpenKeyExW(
            winapi::HKEY_CURRENT_USER,
            wstr!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize").as_ptr(),
            0,
            winapi::KEY_READ | winapi::KEY_WRITE,
            &mut hkey,
        ) == 0 && !hkey.is_null()
        {
            // === 读取当前 SystemUsesLightTheme 值 ===
            let mut current: u32 = 0;
            let mut size: u32 = 4;
            winapi::RegQueryValueExW(
                hkey,
                wstr!("SystemUsesLightTheme").as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut current as *mut _ as *mut u8,
                &mut size,
            );

            // === 切换主题值 ===
            let new_value = 1 - current;

            winapi::RegSetValueExW(
                hkey,
                wstr!("AppsUseLightTheme").as_ptr(),
                0,
                winapi::REG_DWORD,
                &new_value as *const _ as *const u8,
                4,
            );
            winapi::RegSetValueExW(
                hkey,
                wstr!("SystemUsesLightTheme").as_ptr(),
                0,
                winapi::REG_DWORD,
                &new_value as *const _ as *const u8,
                4,
            );
            winapi::RegCloseKey(hkey);
        }

        // === 异步刷新主题（总耗时 <5ms） ===
        // Step 1: 广播主题变更消息（异步）
        notify_all!(winapi::WM_SETTINGCHANGE, wstr!("ImmersiveColorSet").as_ptr() as isize);
        notify_all!(winapi::WM_THEMECHANGED, 0);

        // Step 2: 触发主题服务重载（写回 CurrentTheme）
        let mut hkey_themes: winapi::HKEY = ptr::null_mut();
        if winapi::RegOpenKeyExW(
            winapi::HKEY_CURRENT_USER,
            wstr!(r"Software\Microsoft\Windows\CurrentVersion\Themes").as_ptr(),
            0,
            winapi::KEY_READ | winapi::KEY_WRITE,
            &mut hkey_themes,
        ) == 0 && !hkey_themes.is_null()
        {
            let mut value_buf = [0u16; 520];
            let mut data_size = (value_buf.len() * 2) as u32;
            // 读取原值
            if winapi::RegQueryValueExW(
                hkey_themes,
                wstr!("CurrentTheme").as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                value_buf.as_mut_ptr() as *mut u8,
                &mut data_size,
            ) == 0 && data_size > 0
            {
                // 写回原值
                winapi::RegSetValueExW(
                    hkey_themes,
                    wstr!("CurrentTheme").as_ptr(),
                    0,
                    winapi::REG_SZ,
                    value_buf.as_ptr() as *const u8,
                    data_size - 2,  // 去掉末尾的 null
                );
            }
            winapi::RegCloseKey(hkey_themes);
        }

        // Step 3: 色彩/系统颜色刷新（异步）
        notify_all!(winapi::WM_DWMCOLORIZATIONCOLORCHANGED, 0);

        winapi::ExitProcess(0);
    }
}
