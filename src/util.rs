#[macro_export]
macro_rules! lock {
    ( $mutex_arc:expr ) => {
        $mutex_arc.lock().unwrap()
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
