use std::env;

use std::net::TcpListener;

use tracing_subscriber::filter::LevelFilter;

use tracing::info;

mod patch_pe;

fn main() {

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap_or_else(|error| {
        panic!("Failed to init tracing: {error}")
    });

    let args: Vec<String> = env::args().collect();

    info!("{:?}", args);


    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        use winapi::um::processthreadsapi::ResumeThread;
        use winapi::um::winbase::{CREATE_SUSPENDED, DETACHED_PROCESS};
        use std::os::windows::io::AsRawHandle;
        use dll_syringe::Syringe;
        use dll_syringe::process::{OwnedProcess};

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

        unsafe {
            ResumeThread(syringe.process().as_raw_handle());
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
