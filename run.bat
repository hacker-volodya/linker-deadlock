cargo +stage1 build -Zbuild-std --release --target aarch64-linux-android
adb push .\target\aarch64-linux-android\release\linker-deadlock /data/local/tmp/
adb shell chmod +x /data/local/tmp/linker-deadlock
adb shell /data/local/tmp/linker-deadlock