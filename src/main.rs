use std::env;

use std::net::TcpListener;

use tracing_subscriber::filter::LevelFilter;

use tracing::{info, error};

use clap::Parser;

mod patch_pe;

#[cfg(target_os = "windows")]
use {
    winapi::um::handleapi::CloseHandle,
    winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD},
    winapi::um::processthreadsapi::{OpenThread, ResumeThread},
    winapi::um::winnt::{THREAD_QUERY_INFORMATION, THREAD_SUSPEND_RESUME},
    winapi::shared::minwindef::{DWORD},
    std::process::Command,
    std::os::windows::process::CommandExt,
    winapi::um::winbase::{CREATE_SUSPENDED, DETACHED_PROCESS},
    dll_syringe::Syringe,
    dll_syringe::process::{OwnedProcess, Process},
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "false")]
    resume: bool,
}

fn main() {

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap_or_else(|error| {
        panic!("Failed to init tracing: {error}")
    });

    let args = Args::parse();


    #[cfg(target_os = "windows")]
    {
        info!("Windows");
        const CREATE_FLAGS: u32 = CREATE_SUSPENDED | DETACHED_PROCESS;
        const ZOO_PATH : &str = "C:\\Program Files (x86)\\Microsoft Games\\Zoo Tycoon\\zoo.exe";
        let command: OwnedProcess = Command::new(ZOO_PATH).creation_flags(CREATE_FLAGS).spawn().unwrap().into();

        info!("Process spawned");

        let listener = TcpListener::bind("127.0.0.1:1492").unwrap();

        let syringe = Syringe::for_process(command);
        let _injected_payload = syringe.inject("../openzt/target/i686-pc-windows-msvc/release/openzt.dll").unwrap();

        info!("Dll Injected");

        let mut stream = match listener.accept() {
            Ok((stream, addr)) => {
                info!(%addr, "Accepted connection from");
                stream
            },
            Err(error) => panic!("Log stream failed to connect: {error}")
        };
        
        if args.resume {
            resume_threads(syringe.process().pid().unwrap().into());
        }
        
        info!("Thread Resumed");

        match std::io::copy(&mut stream, &mut std::io::stdout()) {
            Ok(_) => (),
            Err(e) => info!("Logging Stream Closed: {e}")
        };

    }
    #[cfg(not(target_os = "windows"))]
    {
        info!("Not Windows");
    }

}


fn resume_threads(process_id: u32) {
    // Take a snapshot of the threads in the system
    let snap_handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snap_handle.is_null() {
        panic!("Failed to create snapshot: {}", std::io::Error::last_os_error());
    }

    // Enumerate the threads of the process using Thread32First and Thread32Next
    let mut thread_entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    thread_entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
    let result = unsafe { Thread32First(snap_handle, &mut thread_entry) };
    while result != 0 {
        if thread_entry.th32OwnerProcessID == process_id {
            // Open the thread with THREAD_QUERY_INFORMATION and THREAD_SUSPEND_RESUME permissions
            let thread_handle = unsafe {
                OpenThread(
                    THREAD_QUERY_INFORMATION | THREAD_SUSPEND_RESUME,
                    winapi::shared::minwindef::FALSE,
                    thread_entry.th32ThreadID,
                )
            };
            if thread_handle.is_null() {
                error!("Failed to open thread: {}", std::io::Error::last_os_error());
                continue;
            }

            // Resume the thread with ResumeThread
            let result = unsafe { ResumeThread(thread_handle) };
            if result == DWORD::max_value() {
                error!("Failed to resume thread: {}", std::io::Error::last_os_error());
            } else {
                info!("Resumed thread: {}", thread_entry.th32ThreadID);
            }

            // Close the thread handle
            let result = unsafe { CloseHandle(thread_handle) };
            if result == 0 {
                error!("Failed to close thread handle: {}", std::io::Error::last_os_error());
            }
        }

        // Get the next thread in the snapshot
        let result = unsafe { Thread32Next(snap_handle, &mut thread_entry) };
        if result == 0 {
            break;
        }
    }

    // Close the snapshot handle
    let result = unsafe { CloseHandle(snap_handle) };
    if result == 0 {
        panic!("Failed to close snapshot handle: {}", std::io::Error::last_os_error());
    }
}
