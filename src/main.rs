use std::env;
// use std::path::Path;


// use execute::Execute;

mod patch_pe;

fn main() {

    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    // let zoo_directory = Path::new("../zt_files");

    // env::set_current_dir(zoo_directory).unwrap();

    // patch_pe::print_pe().unwrap();

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        use std::os::windows::process::CommandExt;
        use winapi::um::processthreadsapi::ResumeThread;
        use winapi::um::winbase::{CREATE_SUSPENDED, DETACHED_PROCESS};
        use std::os::windows::io::AsRawHandle;
        use dll_syringe::Syringe;
        use dll_syringe::process::{OwnedProcess, Process};

        println!("Windows");
        // const CREATE_SUSPENDED: u32 = 0x00000004;
        // const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_FLAGS: u32 = CREATE_SUSPENDED | DETACHED_PROCESS;
        const ZOO_PATH : &str = "C:\\Program Files (x86)\\Microsoft Games\\Zoo Tycoon\\zoo.exe";
        let command: OwnedProcess = Command::new(ZOO_PATH).creation_flags(CREATE_FLAGS).spawn().unwrap().into();
        // command.creation_flags(CREATE_SUSPENDED);

        // command.spawn().unwrap();

        println!("{CREATE_SUSPENDED} {DETACHED_PROCESS} {CREATE_FLAGS}");
        println!("Process spawned");

        // let mut process: OwnedProcess = command.into();

        // let mut syringe = Syringe::for_process(command.as_handle().as_raw() as u32);
        let syringe = Syringe::for_process(command);
        let injected_payload = syringe.inject("../openzt/target/i686-pc-windows-msvc/release/openzt.dll").unwrap();

        println!("Dll Injected");

        // syringe.eject(injected_payload).unwrap();

    //     unsafe {
    //         ResumeThread(syringe.process().as_raw_handle());
    //     }
        
    //     println!("Thread Resumed");
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("Not Windows");
    }

}
