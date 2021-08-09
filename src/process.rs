// From: https://github.com/mgostIH/process_list

use std::str::{from_utf8, Utf8Error};
use winapi::um::handleapi::CloseHandle;
use winapi::um::winnt::HANDLE;

#[repr(transparent)]
pub(self) struct RAIIHandle(pub HANDLE);

impl RAIIHandle {
    pub fn new(handle: HANDLE) -> RAIIHandle {
        RAIIHandle(handle)
    }
}

impl Drop for RAIIHandle {
    fn drop(&mut self) {
        // This never gives problem except when running under a debugger.
        unsafe { CloseHandle(self.0) };
    }
}

// This is basically from_utf8 with a "transmute" from &[i8] to &[u8]
pub(self) fn get_winstring<'a>(data: &[i8]) -> Result<&'a str, Utf8Error> {
    let len = data.iter().position(|a| *a == 0).unwrap_or(data.len());
    let name: &'a [u8] = unsafe { std::slice::from_raw_parts(data.as_ptr().cast(), len) };
    from_utf8(name)
}

use std::io;
use std::path::Path;
use winapi::shared::minwindef::TRUE;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};


/// Computes a function for each process found.
///
/// The function `f` takes the process id and it's name as parameters and can do whatever needed.
///
/// Processes that have an invalid UTF-8 name are ignored and logged in warn level (May change in the future)
///
/// # Returns
/// This function returns the error if any of the internal WinAPI fails.
///
/// # Examples
/// Printing every process to `stdout`
/// ```
/// use std::path::{Path, PathBuf};
/// use process_list::for_each_process;
/// fn print_processes(id : u32, name : &Path) {
///     println!("Id: {} --- Name: {}", id, name.display());
/// }
///
/// for_each_process(print_processes).unwrap();
/// ```
///
/// # Examples
/// Getting all the processes into a `Vec`
/// ```
/// use std::path::{Path, PathBuf};
/// use process_list::for_each_process;
/// let mut data : Vec<(u32, PathBuf)> = Vec::new();
/// for_each_process(|id, name| data.push( (id, name.to_path_buf()) )).unwrap();
/// // Now `data` holds all the current processes id-name pairs.
/// ```
pub fn for_each_process<F>(mut f: F) -> io::Result<()>
    where
        F: FnMut(u32, &Path),
{
    // Safe, we need to interface with WinAPI, there's not particular preconditions for the input.
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if handle == INVALID_HANDLE_VALUE {
        let error = io::Error::last_os_error();
        return Err(error);
    }
    let _guard = RAIIHandle::new(handle); // We don't actually use this but we want to call CloseHandle when we are done

    // Safe because it's a WINAPI type, using MaybeUninit would be hard because we need to write on its dwSize field.
    let mut pe32: PROCESSENTRY32 = unsafe { std::mem::zeroed() };
    // We must initialize dwSize here.
    pe32.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

    let v = unsafe { Process32First(handle, &mut pe32) };
    if v != TRUE {
        let error = io::Error::last_os_error();
        return Err(error);
    }
    loop {
        match get_process_data(&pe32) {
            Ok((id, name)) => f(id, name),
            // Don't change the underscore from id
            Err(_id) => {},
        }

        // Cleans back the storage we used to store the process name.
        pe32.szExeFile
            .iter_mut()
            .take_while(|c| **c != 0)
            .for_each(|c| *c = 0);

        if unsafe { Process32Next(handle, &mut pe32) } == FALSE {
            break
        }
    }

    // No need to call CloseHandle as it's dealt by the RAIIHandle.
    Ok(())
}

fn get_process_data(process: &PROCESSENTRY32) -> Result<(u32, &Path), u32> {
    let id = process.th32ProcessID;
    let name = get_winstring(&process.szExeFile).map_err(|_| id)?;
    Ok((id, Path::new(name)))
}