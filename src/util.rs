use log::error;
#[macro_export]
macro_rules! lock {
    ( $mutex_arc:expr ) => {
        $mutex_arc.lock().unwrap()
        // if let Ok(res) = $mutex_arc.lock() {
        //     return Ok(res);
        // } else {
        //     error!("unable to get lock on mutex");
        //     return Err("unable to get lock on mutex");
        // }
    };
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::network::types::ArcMut;

    use super::*;

    #[test]
    fn test_lock_macro() {
        let data = vec![1, 2, 3];

        let arc = ArcMut::new(data);

        let mut lock = lock!(arc);

        lock.push(4);

        assert_eq!(format!("{:?}", [1, 2, 3, 4]), format!("{lock:?}"));
    }
}
