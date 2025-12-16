#![windows_subsystem = "windows"]

use std::{
    ffi::OsStr,
    os::windows::ffi::OsStrExt,
    ptr,
};

use windows_sys::Win32::{
    System::Registry::{
        RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, RegCloseKey,
        HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_DWORD,
    },
    UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG,
        WM_SETTINGCHANGE, WM_THEMECHANGED, WM_SYSCOLORCHANGE,
    },
};

const PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

// 转 UTF-16（W 系列 API 必须用 UTF-16）
fn to_utf16(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

unsafe fn read_reg_dword(root: HKEY, path: &str, name: &str) -> Option<u32> {
    let path_w = to_utf16(path);
    let name_w = to_utf16(name);

    let mut hkey: HKEY = std::ptr::null_mut();
    if RegOpenKeyExW(root, path_w.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
        return None;
    }

    let mut data: u32 = 0;
    let mut data_size = std::mem::size_of::<u32>() as u32;

    let ok = RegQueryValueExW(
        hkey,
        name_w.as_ptr(),
        ptr::null_mut(),
        ptr::null_mut(),
        &mut data as *mut _ as *mut u8,
        &mut data_size,
    );

    RegCloseKey(hkey);

    if ok == 0 {
        Some(data)
    } else {
        None
    }
}

unsafe fn write_reg_dword(root: HKEY, path: &str, name: &str, value: u32) -> bool {
    let path_w = to_utf16(path);
    let name_w = to_utf16(name);

    // RegOpenKeyExW 可替代 CreateKeyW——若不存在会自动创建
    let mut hkey: HKEY = std::ptr::null_mut();
    if RegOpenKeyExW(root, path_w.as_ptr(), 0, KEY_WRITE, &mut hkey) != 0 {
        return false;
    }

    let res = RegSetValueExW(
        hkey,
        name_w.as_ptr(),
        0,
        REG_DWORD,
        &value as *const _ as *const u8,
        std::mem::size_of::<u32>() as u32,
    );

    RegCloseKey(hkey);
    res == 0
}

// WM 广播
unsafe fn broadcast(msg: u32, lparam: isize) {
    SendMessageTimeoutW(
        HWND_BROADCAST,
        msg,
        0,
        lparam,
        SMTO_ABORTIFHUNG,
        100,
        std::ptr::null_mut(),
    );
}

// 刷新系统主题
unsafe fn refresh_theme() {
    let theme_str = to_utf16("ImmersiveColorSet");
    broadcast(WM_SETTINGCHANGE, theme_str.as_ptr() as isize);
    
    // 目前实测发现 WM_THEMECHANGED 和 WM_SYSCOLORCHANGE 的广播非必须，暂时注释掉（调试用）
    // broadcast(WM_THEMECHANGED, 0);
    // broadcast(WM_SYSCOLORCHANGE, 0);
}

fn main() {
    unsafe {
        let current = read_reg_dword(HKEY_CURRENT_USER, PATH, "AppsUseLightTheme").unwrap_or(1);
        // let new_value = if current == 0 { 1 } else { 0 };
        let new_value = 1 - current; // 更简洁的切换逻辑

        write_reg_dword(HKEY_CURRENT_USER, PATH, "AppsUseLightTheme", new_value);
        write_reg_dword(HKEY_CURRENT_USER, PATH, "SystemUsesLightTheme", new_value);

        refresh_theme();
    }
}
