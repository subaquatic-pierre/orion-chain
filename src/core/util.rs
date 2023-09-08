use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp(time: SystemTime) -> u64 {
    let since_the_epoch = time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}
