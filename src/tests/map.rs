use crate::{map::sanitize_key, shm::shm_open_read, Shmap};
use memmap2::Mmap;
use rand::{distributions::Alphanumeric, prelude::SliceRandom, thread_rng, Rng};
use std::time::Duration;

pub fn rand_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn read_from_shm(sanitized_key: &str) -> Vec<u8> {
    let fd = shm_open_read(sanitized_key).unwrap();
    let mmap = unsafe { Mmap::map(fd) }.unwrap();
    mmap.to_vec()
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
fn test_different_size() {
    let key = rand_string(10);

    let shmap = Shmap::new();
    let value = rand_string(50);
    shmap.insert(&key, value.to_owned()).unwrap();
    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let shmap = Shmap::new();
    let value = rand_string(100);
    shmap.insert(&key, value.to_owned()).unwrap();
    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let shmap = Shmap::new();
    let value = rand_string(20);
    shmap.insert(&key, value.to_owned()).unwrap();
    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    shmap.remove(&key).unwrap();
}

#[test]
fn test_encrypted() {
    let mut secret: Vec<u8> = (0..32).collect();
    secret.shuffle(&mut thread_rng());

    let shmap_enc = Shmap::new_with_encryption(&secret.try_into().unwrap());
    let key = rand_string(10);
    let value = rand_string(50);

    shmap_enc.insert(&key, value.to_owned()).unwrap();
    let ret_value_1: String = shmap_enc.get(&key).unwrap().unwrap();
    assert_eq!(ret_value_1, value);

    // Compare with non-encrypted
    let shmap = Shmap::new();
    let key_2 = rand_string(10);
    shmap.insert(&key_2, value.to_owned()).unwrap();
    let ret_value_2: String = shmap.get(&key_2).unwrap().unwrap();
    assert_eq!(ret_value_2, value);
    assert_eq!(ret_value_1, ret_value_2);
    let raw_1 = read_from_shm(&sanitize_key(&key));
    let raw_2 = read_from_shm(&sanitize_key(&key_2));
    assert_ne!(raw_1, raw_2);

    shmap_enc.remove(&key).unwrap();
    shmap.remove(&key_2).unwrap();
}

#[test]
#[should_panic(expected = "AesGcmError(Error)")]
fn test_bad_key() {
    let key = rand_string(10);
    let value = rand_string(50);

    let mut secret: Vec<u8> = (0..32).collect();
    secret.shuffle(&mut thread_rng());
    let shmap = Shmap::new_with_encryption(&secret.try_into().unwrap());
    shmap.insert(&key, value.to_owned()).unwrap();
    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let mut secret: Vec<u8> = (0..32).collect();
    secret.shuffle(&mut thread_rng());
    let shmap = Shmap::new_with_encryption(&secret.try_into().unwrap());
    let _: String = shmap.get(&key).unwrap().unwrap();

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

#[test]
#[should_panic(expected = "Option::unwrap()")]
fn test_expiration() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let value = rand_string(50);

    shmap
        .insert_with_ttl(&key, value.to_owned(), Duration::from_millis(200))
        .unwrap();
    shmap.clean().unwrap();
    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    std::thread::sleep(Duration::from_millis(300));

    let _: String = shmap.get(&key).unwrap().unwrap();
}

#[test]
fn test_many_fd() {
    let shmap = Shmap::new();

    // set fd limit to 42 for testing purpose
    unsafe {
        let rlim: libc::rlimit = libc::rlimit {
            rlim_cur: 42,
            rlim_max: 42,
        };
        if libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) != 0 {
            let err = std::io::Error::last_os_error();
            panic!("raise_fd_limit: error calling setrlimit: {}", err);
        }
    }

    for i in 0..50 {
        let key = rand_string(i);
        shmap.insert(&key, "0").unwrap();
    }

    fdlimit::raise_fd_limit();
}

// test concurrency between set
#[test]
fn test_set_concurrency() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let key_clone = key.clone();

    let shmap_clone = shmap.clone();
    let task = move || {
        for i in 0..1024 {
            let value = rand_string(i);
            shmap_clone.insert(&key, value).unwrap();
        }
    };

    let t1 = std::thread::spawn(task.clone());
    let t2 = std::thread::spawn(task);

    t1.join().unwrap();
    t2.join().unwrap();

    shmap.remove(&key_clone).unwrap();
}

// test concurrency between get
#[test]
fn test_get_concurrency() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let value = rand_string(50);
    let key_clone = key.clone();

    shmap.insert(&key, value).unwrap();

    let shmap_clone = shmap.clone();
    let task = move || {
        for _ in 0..1024 {
            let _: String = shmap_clone.get(&key).unwrap().unwrap();
        }
    };

    let t1 = std::thread::spawn(task.clone());
    let t2 = std::thread::spawn(task);

    t1.join().unwrap();
    t2.join().unwrap();

    shmap.remove(&key_clone).unwrap();
}

// test concurrency between set and get
#[test]
fn test_get_set_concurrency() {
    let shmap = Shmap::new();
    let key = rand_string(10);
    let key_clone = key.clone();

    let shmap_clone = shmap.clone();
    let task = move || {
        for i in 0..1024 {
            let value = rand_string(i);
            shmap_clone.insert(&key, value.to_owned()).unwrap();
            let _: String = shmap_clone.get(&key).unwrap().unwrap();
        }
    };

    let t1 = std::thread::spawn(task.clone());
    let t2 = std::thread::spawn(task);

    t1.join().unwrap();
    t2.join().unwrap();

    shmap.remove(&key_clone).unwrap();
}

// test concurrency with metadatas set/remove
#[test]
fn test_metadatas_concurrency() {
    let key = rand_string(10);

    let task = move || {
        for i in 0..1024 {
            let shmap = Shmap::new();
            let value = rand_string(i);
            shmap.insert(&key, value.to_owned()).unwrap();
            let _: Option<String> = shmap.get(&key).unwrap();
            shmap.remove(&key).unwrap();
        }
    };

    let t1 = std::thread::spawn(task.clone());
    let t2 = std::thread::spawn(task);

    t1.join().unwrap();
    t2.join().unwrap();
}
