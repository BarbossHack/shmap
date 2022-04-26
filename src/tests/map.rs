use crate::{get, remove, set};
use rand::{distributions::Alphanumeric, Rng};
use std::time::Duration;

fn rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[test]
#[should_panic(expected = "Option::unwrap()")]
fn test_get_unknown() {
    let key = rand_string(10);
    let _: String = get(&key).unwrap().unwrap();
}

#[test]
fn test_set_and_get() {
    let key = rand_string(10);
    let value = rand_string(50);

    set(&key, value.to_owned()).unwrap();

    let ret_value: String = get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let ret_value: String = get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    remove(&key).unwrap();

    let key = rand_string(10);
    let value = vec!["Test".to_string(), "Vec".to_string()];

    set(&key, value.to_owned()).unwrap();

    let ret_value: Vec<String> = get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let ret_value: Vec<String> = get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    remove(&key).unwrap();
}

#[test]
fn test_set_and_get_big() {
    let key = rand_string(10);
    let value = rand_string(5 * 1024 * 1024);

    set(&key, value.to_owned()).unwrap();

    let ret_value: String = get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let ret_value: String = get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    remove(&key).unwrap();
}

#[test]
fn test_remove() {
    let key = rand_string(10);
    let value = rand_string(50);

    set(&key, value.to_owned()).unwrap();

    remove(&key).unwrap();
}

#[test]
#[should_panic(expected = "ShmUnlinkFailed(-1)")]
fn test_remove_unknown() {
    let key = rand_string(10);
    remove(&key).unwrap();
}

// test_namedlock_set_1() and test_namedlock_set_2() may fail (as run in parallel) without a proper
// inter process Lock (named_lock here)
#[test]
fn test_namedlock_set_1() {
    let key = "test_namedlock_set";

    for i in 0..1024 {
        let value = rand_string(i);
        set(key, value).unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = remove(&key);
}

#[test]
fn test_namedlock_set_2() {
    let key = "test_namedlock_set";

    for i in 0..1024 {
        let value = rand_string(i);
        set(key, value).unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = remove(&key);
}

// test_namedlock_get_1() and test_namedlock_get_2() should not fail even with inter process Lock
// if there is no set()
#[test]
fn test_namedlock_get_1() {
    let key = "test_namedlock_get";
    let value = rand_string(50);
    set(key, value.to_owned()).unwrap();

    std::thread::sleep(Duration::from_millis(100));

    for _ in 0..1024 {
        let _: String = get(key).unwrap().unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = remove(&key);
}

#[test]
fn test_namedlock_get_2() {
    let key = "test_namedlock_get";
    let value = rand_string(50);
    set(key, value.to_owned()).unwrap();

    std::thread::sleep(Duration::from_millis(100));

    for _ in 0..1024 {
        let _: String = get(key).unwrap().unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = remove(&key);
}

// test_namedlock_get_set_1() and test_namedlock_get_set_2() may fail (as run in parallel) without a proper
// inter process Lock (named_lock here), in set() AND get()
#[test]
fn test_namedlock_get_set_1() {
    let key = "test_namedlock_get_set";

    for i in 0..1024 {
        let value = rand_string(i);
        set(key, value.to_owned()).unwrap();
        let _: String = get(key).unwrap().unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = remove(&key);
}

#[test]
fn test_namedlock_get_set_2() {
    let key = "test_namedlock_get_set";

    for i in 0..1024 {
        let value = rand_string(i);
        set(key, value.to_owned()).unwrap();
        let _: String = get(key).unwrap().unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    let _ = remove(&key);
}
