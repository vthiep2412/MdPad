use std::path::PathBuf;

#[cfg(windows)]
#[repr(C)]
#[allow(non_snake_case)]
struct OPENFILENAMEW {
    lStructSize: u32,
    hwndOwner: *mut std::ffi::c_void,
    hInstance: *mut std::ffi::c_void,
    lpstrFilter: *const u16,
    lpstrCustomFilter: *mut u16,
    nMaxCustFilter: u32,
    nFilterIndex: u32,
    lpstrFile: *mut u16,
    nMaxFile: u32,
    lpstrFileTitle: *mut u16,
    nMaxFileTitle: u32,
    lpstrInitialDir: *const u16,
    lpstrTitle: *const u16,
    Flags: u32,
    nFileOffset: u16,
    nFileExtension: u16,
    lpstrDefExt: *const u16,
    lCustData: usize,
    lpfnHook: *mut std::ffi::c_void,
    lpTemplateName: *const u16,
    pvReserved: *mut std::ffi::c_void,
    dwReserved: u32,
    FlagsEx: u32,
}

#[cfg(windows)]
#[link(name = "comdlg32")]
extern "system" {
    fn GetOpenFileNameW(lpofn: *mut OPENFILENAMEW) -> i32;
    fn GetSaveFileNameW(lpofn: *mut OPENFILENAMEW) -> i32;
}

#[cfg(windows)]
fn encode_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn open_file_dialog() -> Option<PathBuf> {
    #[cfg(windows)]
    unsafe {
        let mut file_buf = vec![0u16; 1024];
        let filter: Vec<u16> = "Markdown Files (*.md, *.markdown)\0*.md;*.markdown\0Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0\0"
            .encode_utf16()
            .collect();
        let title = encode_wide("Open Markdown Document");

        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.lpstrFilter = filter.as_ptr();
        ofn.nFilterIndex = 1;
        ofn.lpstrFile = file_buf.as_mut_ptr();
        ofn.nMaxFile = file_buf.len() as u32;
        ofn.lpstrTitle = title.as_ptr();
        ofn.Flags = 0x00001000 | 0x00000800 | 0x00080000;

        if GetOpenFileNameW(&mut ofn) != 0 {
            let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
            let path_str = String::from_utf16_lossy(&file_buf[..len]);
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
        None
    }

    #[cfg(not(windows))]
    {
        None
    }
}

pub fn save_file_dialog(default_name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    unsafe {
        let mut file_buf = vec![0u16; 1024];
        let def_wide = encode_wide(default_name);
        for (i, &c) in def_wide.iter().enumerate() {
            if i < file_buf.len() - 1 {
                file_buf[i] = c;
            }
        }

        let filter: Vec<u16> = "Markdown Files (*.md)\0*.md\0Text Files (*.txt)\0*.txt\0All Files (*.*)\0*.*\0\0"
            .encode_utf16()
            .collect();
        let title = encode_wide("Save Markdown Document");
        let def_ext = encode_wide("md");

        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.lpstrFilter = filter.as_ptr();
        ofn.nFilterIndex = 1;
        ofn.lpstrFile = file_buf.as_mut_ptr();
        ofn.nMaxFile = file_buf.len() as u32;
        ofn.lpstrTitle = title.as_ptr();
        ofn.lpstrDefExt = def_ext.as_ptr();
        ofn.Flags = 0x00000002 | 0x00000800 | 0x00080000;

        if GetSaveFileNameW(&mut ofn) != 0 {
            let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
            let path_str = String::from_utf16_lossy(&file_buf[..len]);
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
        None
    }

    #[cfg(not(windows))]
    {
        let _ = default_name;
        None
    }
}
