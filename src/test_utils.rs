use std::sync::Mutex;

static CWD_LOCK: Mutex<()> = Mutex::new(());

pub fn with_cwd_locked<T>(dir: &std::path::Path, f: impl FnOnce() -> T) -> T {
    let _lock = CWD_LOCK.lock().unwrap();
    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir).unwrap();
    let result = f();
    std::env::set_current_dir(&original).unwrap();
    result
}
