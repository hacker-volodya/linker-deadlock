# linker-deadlock
Deadlock example for the https://github.com/rust-lang/rust/issues/88585 bug

## Description
There is a problem with `Command::spawn` on Android. When another thread interacts with system linker, `Command::spawn` causes a deadlock in the child process between `fork` and `exec`. It happens because under the hood Android linker uses global mutex `g_dl_mutex` ([link to android source](https://android.googlesource.com/platform/bionic/+/563e60e32adf7f427af03075a6fb800e723025f1/linker/dlfcn.cpp#97)) and not doing `pthread_atfork()` to unlock it in child processes.

What exactly happens:

1. [Command::spawn](https://github.com/rust-lang/rust/blob/cc9bb1522e357a4a11e7b0bfbbb7eddbd880a44f/library/std/src/sys/unix/process/process_unix.rs#L37) is called.
2. `Command::spawn` actually invokes `do_fork`, then `do_exec` is invoked in the child process.
3. There is a comment in [do_exec](https://github.com/rust-lang/rust/blob/cc9bb1522e357a4a11e7b0bfbbb7eddbd880a44f/library/std/src/sys/unix/process/process_unix.rs#L246-L275), which says that a deadlock may occur, so no allocations must appear in `do_exec`.
4. Well, at this point we know that any code between `fork` and `exec` must not lock mutexes from the parent process, because it may be "forever locked" bacause of `fork`.
5. `do_exec` calls `sys::signal`. Fine.
6. Seems interesting that [sys::signal](https://github.com/rust-lang/rust/blob/cc9bb1522e357a4a11e7b0bfbbb7eddbd880a44f/library/std/src/sys/unix/android.rs#L76) is linking actual `signal` function from libc using [dlsym](https://github.com/rust-lang/rust/blob/cc9bb1522e357a4a11e7b0bfbbb7eddbd880a44f/library/std/src/sys/unix/weak.rs#L101).
7. At this point linker is [trying to lock](https://android.googlesource.com/platform/bionic/+/563e60e32adf7f427af03075a6fb800e723025f1/linker/dlfcn.cpp#159) parent mutex [g_dl_mutex](https://android.googlesource.com/platform/bionic/+/563e60e32adf7f427af03075a6fb800e723025f1/linker/dlfcn.cpp#97), which may be in "forever locked" state, so the child process hangs.

`main.rs`:
```rust
#![no_main]

#[link(name = "dl")]
extern {
    fn dlsym(handle: i32, symbol: i32);
}

/// Implementing here raw main function, because Rust initializers calls `signal` and caches its address, 
/// preventing deadlock. In real life Rust not always compiles to standalone executables, so this case
/// is also applicable to shared libraries. 
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
```

How to build and run (can run on real device or emulator, to run on emulator replace `aarch64` with `x86_64`):
```bash
cargo build --release --target aarch64-linux-android
adb push .\target\aarch64-linux-android\release\linker-deadlock /data/local/tmp/
adb shell chmod +x /data/local/tmp/linker-deadlock
adb shell /data/local/tmp/linker-deadlock
```

be sure to specify Android NDK toolchain binaries in `~/.cargo/config.toml`:
```toml
[target.aarch64-linux-android]
ar = "C:\\Users\\user\\AppData\\Local\\Android\\Sdk\\ndk\\20.0.5594570\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\llvm-ar.exe"
linker = "C:\\Users\\user\\AppData\\Local\\Android\\Sdk\\ndk\\20.0.5594570\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\aarch64-linux-android23-clang.cmd"

[target.x86_64-linux-android]
ar = "C:\\Users\\user\\AppData\\Local\\Android\\Sdk\\ndk\\20.0.5594570\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\llvm-ar.exe"
linker = "C:\\Users\\user\\AppData\\Local\\Android\\Sdk\\ndk\\20.0.5594570\\toolchains\\llvm\\prebuilt\\windows-x86_64\\bin\\x86_64-linux-android23-clang.cmd"
```

## Meta

`rustc --version --verbose`:
```
rustc 1.54.0 (a178d0322 2021-07-26)
binary: rustc
commit-hash: a178d0322ce20e33eac124758e837cbd80a6f633
commit-date: 2021-07-26
host: x86_64-pc-windows-msvc
release: 1.54.0
LLVM version: 12.0.1
```

```
rustc 1.56.0-nightly (50171c310 2021-09-01)
binary: rustc
commit-hash: 50171c310cd15e1b2d3723766ce64e2e4d6696fc
commit-date: 2021-09-01
host: x86_64-pc-windows-msvc
release: 1.56.0-nightly
LLVM version: 13.0.0
```

</p>
</details>
