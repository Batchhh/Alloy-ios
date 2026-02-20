#[macro_export]
macro_rules! loop_thread {
    ($seconds:expr, $body:block) => {
        std::thread::spawn(move || loop {
            $body;
            std::thread::sleep(std::time::Duration::from_secs($seconds));
        });
    };
    ($($body:tt)*) => {
        std::thread::spawn(move || loop {
            $($body)*;
            std::thread::sleep(std::time::Duration::from_millis(33)); // 30 FPS
        });
    };
}
