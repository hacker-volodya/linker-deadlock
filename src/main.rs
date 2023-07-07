#![no_main]

#[link(name = "dl")]
extern {
    fn dlsym(handle: i32, symbol: i32);
}

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let t = std::thread::spawn(|| {
        println!("Locking g_dl_mutex...");
        for _ in 0..10000000 {
            unsafe { dlsym(0, 0); }
        }
        println!("dlsym'ing done");
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("Spawning ps...");
    let _child = std::process::Command::new("ps").spawn().unwrap();
    println!("ps spawned!");
    t.join().unwrap();
    println!("Done");
    0
}