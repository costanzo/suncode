//! Preserve access permissions and publish with a single handle-based rename.
//! POSIX rename semantics keep existing readers on the old complete version.
use std::{io, os::windows::ffi::OsStrExt, path::Path};
use windows_sys::Win32::Security::{
    GetFileSecurityW, GetSecurityDescriptorControl, SetFileSecurityW, DACL_SECURITY_INFORMATION,
    PROTECTED_DACL_SECURITY_INFORMATION, SE_DACL_PROTECTED, UNPROTECTED_DACL_SECURITY_INFORMATION,
};

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}

fn descriptor(path: &Path) -> io::Result<Vec<u32>> {
    let path = wide(path);
    let mut needed = 0u32;
    // Query the size, then allocate aligned storage for the self-relative SD.
    unsafe {
        GetFileSecurityW(
            path.as_ptr(),
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
    }
    if needed == 0 || needed > 1024 * 1024 {
        return Err(io::Error::last_os_error());
    }
    let mut buffer = vec![0u32; (needed as usize).div_ceil(4)];
    let success = unsafe {
        GetFileSecurityW(
            path.as_ptr(),
            DACL_SECURITY_INFORMATION,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    };
    if success == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(buffer)
}

pub(super) fn copy_dacl(source: &Path, destination: &Path) -> io::Result<()> {
    let mut descriptor = descriptor(source)?;
    let pointer = descriptor.as_mut_ptr().cast();
    let mut control = 0u16;
    let mut revision = 0u32;
    // All pointers refer to live, aligned buffers for the duration of each call.
    if unsafe { GetSecurityDescriptorControl(pointer, &mut control, &mut revision) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let inheritance = if control & SE_DACL_PROTECTED != 0 {
        PROTECTED_DACL_SECURITY_INFORMATION
    } else {
        UNPROTECTED_DACL_SECURITY_INFORMATION
    };
    let destination = wide(destination);
    if unsafe {
        SetFileSecurityW(
            destination.as_ptr(),
            DACL_SECURITY_INFORMATION | inheritance,
            pointer,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(super) fn replace(staged: tempfile::NamedTempFile, destination: &Path) -> io::Result<()> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        FileRenameInfoEx, SetFileAttributesW, SetFileInformationByHandle, DELETE,
        FILE_ATTRIBUTE_NORMAL, FILE_RENAME_INFO,
    };
    use windows_sys::Win32::System::WindowsProgramming::{
        FILE_RENAME_FLAG_POSIX_SEMANTICS, FILE_RENAME_FLAG_REPLACE_IF_EXISTS,
    };
    let temporary = staged.into_temp_path();
    let source = wide(&temporary);
    if unsafe { SetFileAttributesW(source.as_ptr(), FILE_ATTRIBUTE_NORMAL) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let handle = std::fs::OpenOptions::new()
        .access_mode(DELETE)
        .open(&temporary)?;
    let name = wide(destination);
    let name_bytes = (name.len() - 1) * 2;
    let size = (std::mem::offset_of!(FILE_RENAME_INFO, FileName) + name_bytes)
        .max(std::mem::size_of::<FILE_RENAME_INFO>());
    let mut buffer = vec![0usize; size.div_ceil(std::mem::size_of::<usize>())];
    let information = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    // Aligned zeroed storage includes the flexible trailing UTF-16 name.
    // RootDirectory stays null; the canonical project supplies an absolute name.
    unsafe {
        (*information).Anonymous.Flags =
            FILE_RENAME_FLAG_REPLACE_IF_EXISTS | FILE_RENAME_FLAG_POSIX_SEMANTICS;
        (*information).FileNameLength = name_bytes as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            std::ptr::addr_of_mut!((*information).FileName).cast::<u16>(),
            name.len() - 1,
        );
    }
    let success = unsafe {
        SetFileInformationByHandle(
            handle.as_raw_handle().cast(),
            FileRenameInfoEx,
            information.cast(),
            size as u32,
        )
    };
    if success == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replacement_preserves_protected_dacl() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().canonicalize().unwrap().join("source");
        std::fs::write(&source, b"before").unwrap();
        let mut original = descriptor(&source).unwrap();
        assert_ne!(
            unsafe {
                SetFileSecurityW(
                    wide(&source).as_ptr(),
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    original.as_mut_ptr().cast(),
                )
            },
            0
        );
        let expected = descriptor(&source).unwrap();
        let staged = super::super::stage(&source, b"after", true).unwrap();
        super::super::replace(staged, &source).unwrap();
        assert_eq!(descriptor(&source).unwrap(), expected);
        assert_eq!(std::fs::read(&source).unwrap(), b"after");
    }
}
