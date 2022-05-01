use crate::Shmap;
use rand::{distributions::Alphanumeric, Rng};
use std::time::Duration;

pub fn rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[test]
#[should_panic(expected = "Option::unwrap()")]
fn test_get_unknown() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let _: String = shmap.get(&key).unwrap().unwrap();
}

#[test]
fn simple_test() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let value = rand_string(50);

    shmap.insert(&key, value.to_owned()).unwrap();
    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);
    shmap.remove(&key).unwrap();
}

#[test]
fn test_set_and_get() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let value = rand_string(50);

    shmap.insert(&key, value.to_owned()).unwrap();

    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    shmap.remove(&key).unwrap();

    let key = rand_string(10);
    let value = vec!["Test".to_string(), "Vec".to_string()];

    shmap.insert(&key, value.to_owned()).unwrap();

    let ret_value: Vec<String> = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let ret_value: Vec<String> = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    shmap.remove(&key).unwrap();
}

#[test]
fn test_set_and_get_big() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let value = rand_string(5 * 1024 * 1024);

    shmap.insert(&key, value.to_owned()).unwrap();

    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    shmap.remove(&key).unwrap();
}

#[test]
fn test_remove() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let value = rand_string(50);

    shmap.insert(&key, value).unwrap();

    shmap.remove(&key).unwrap();
}

#[test]
fn test_remove_not_found() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    shmap.remove(&key).unwrap();
}

// test_namedlock_set_1() and test_namedlock_set_2() may fail (as run in parallel) without a proper
// inter process Lock (named_lock here)
#[test]
fn test_namedlock_set_1() {
    let shmap = Shmap::new();
    let key = "test_namedlock_set";

    for i in 0..1024 {
        let value = rand_string(i);
        shmap.insert(key, value).unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = shmap.remove(&key);
}

#[test]
fn test_namedlock_set_2() {
    let shmap = Shmap::new();
    let key = "test_namedlock_set";

    for i in 0..1024 {
        let value = rand_string(i);
        shmap.insert(key, value).unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = shmap.remove(&key);
}

// test_namedlock_get_1() and test_namedlock_get_2() should not fail even with inter process Lock
// if there is no set()
#[test]
fn test_namedlock_get_1() {
    let shmap = Shmap::new();
    let key = "test_namedlock_get";
    let value = rand_string(50);
    shmap.insert(key, value.to_owned()).unwrap();

    std::thread::sleep(Duration::from_millis(100));

    for _ in 0..1024 {
        let _: String = shmap.get(key).unwrap().unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = shmap.remove(&key);
}

#[test]
fn test_namedlock_get_2() {
    let shmap = Shmap::new();
    let key = "test_namedlock_get";
    let value = rand_string(50);
    shmap.insert(key, value.to_owned()).unwrap();

    std::thread::sleep(Duration::from_millis(100));

    for _ in 0..1024 {
        let _: String = shmap.get(key).unwrap().unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = shmap.remove(&key);
}

// test_namedlock_get_set_1() and test_namedlock_get_set_2() may fail (as run in parallel) without a proper
// inter process Lock (named_lock here), in set() AND get()
#[test]
fn test_namedlock_get_set_1() {
    let shmap = Shmap::new();
    let key = "test_namedlock_get_set";

    for i in 0..1024 {
        let value = rand_string(i);
        shmap.insert(key, value.to_owned()).unwrap();
        let _: String = shmap.get(key).unwrap().unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = shmap.remove(&key);
}

#[test]
fn test_namedlock_get_set_2() {
    let shmap = Shmap::new();
    let key = "test_namedlock_get_set";

    for i in 0..1024 {
        let value = rand_string(i);
        shmap.insert(key, value.to_owned()).unwrap();
        let _: String = shmap.get(key).unwrap().unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = shmap.remove(&key);
}

// test concurrency with indexes set/remove
#[test]
fn test_indexes_concurrency_1() {
    let key = "test_indexes_concurrency";

    for i in 0..1024 {
        let shmap = Shmap::new();
        let value = rand_string(i);
        shmap.insert(key, value.to_owned()).unwrap();
        shmap.remove(key).unwrap();
    }
}

#[test]
fn test_indexes_concurrency_2() {
    let key = "test_indexes_concurrency";

    for i in 0..1024 {
        let shmap = Shmap::new();
        let value = rand_string(i);
        shmap.insert(key, value.to_owned()).unwrap();
        shmap.remove(key).unwrap();
    }
}
