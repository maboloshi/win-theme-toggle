#![windows_subsystem = "windows"]

use std::ptr;
use windows_sys::Win32::{
    System::Registry::{
        RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, RegCloseKey,
        HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_DWORD, REG_SZ,
    },
    UI::WindowsAndMessaging::{
        SendNotifyMessageW, HWND_BROADCAST, WM_SETTINGCHANGE, WM_THEMECHANGED,
    },
};

const WM_DWMCOLORIZATIONCOLORCHANGED: u32 = 0x0320;

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

unsafe fn notify_all(msg: u32, lparam: isize) {
    SendNotifyMessageW(HWND_BROADCAST, msg, 0, lparam);
}

fn main() {
    unsafe {
        // === 读取、反转当前主题值（基于 SystemUsesLightTheme） ===
        let mut hkey: HKEY = ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wstr!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize").as_ptr(),
            0,
            KEY_READ | KEY_WRITE,
            &mut hkey,
        ) == 0 && !hkey.is_null()
        {
            // === 读取当前 SystemUsesLightTheme 值 ===
            let mut current: u32 = 0;
            let mut size = std::mem::size_of::<u32>() as u32;
            RegQueryValueExW(
                hkey,
                wstr!("SystemUsesLightTheme").as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut current as *mut _ as *mut u8,
                &mut size,
            );

            // === 切换主题值 ===
            let new_value = 1 - current;

            RegSetValueExW(
                hkey,
                wstr!("AppsUseLightTheme").as_ptr(),
                0,
                REG_DWORD,
                &new_value as *const _ as *const u8,
                4,
            );
            RegSetValueExW(
                hkey,
                wstr!("SystemUsesLightTheme").as_ptr(),
                0,
                REG_DWORD,
                &new_value as *const _ as *const u8,
                4,
            );
            RegCloseKey(hkey);
        }

        // === 异步刷新主题（总耗时 <5ms） ===
        // Step 1: 广播主题变更消息（异步）
        notify_all(WM_SETTINGCHANGE, wstr!("ImmersiveColorSet").as_ptr() as isize);
        notify_all(WM_THEMECHANGED, 0);

        // Step 2: 触发主题服务重载（写回 CurrentTheme）
        let mut hkey_themes: HKEY = ptr::null_mut();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            wstr!(r"Software\Microsoft\Windows\CurrentVersion\Themes").as_ptr(),
            0,
            KEY_READ | KEY_WRITE,
            &mut hkey_themes,
        ) == 0 && !hkey_themes.is_null()
        {
            let mut value_buf = [0u16; 520];
            let mut data_size = (value_buf.len() * 2) as u32;
            if RegQueryValueExW(
                hkey_themes,
                wstr!("CurrentTheme").as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                value_buf.as_mut_ptr() as *mut u8,
                &mut data_size,
            ) == 0 && data_size > 0
            {
                // 写回原值
                RegSetValueExW(
                    hkey_themes,
                    wstr!("CurrentTheme").as_ptr(),
                    0,
                    REG_SZ,
                    value_buf.as_ptr() as *const u8,
                    data_size - 2, // 去掉末尾的 null
                );
            }
            RegCloseKey(hkey_themes);
        }

        // Step 3: 色彩/系统颜色刷新（异步）
        notify_all(WM_DWMCOLORIZATIONCOLORCHANGED, 0);
    }
}