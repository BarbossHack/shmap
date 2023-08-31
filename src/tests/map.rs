use crate::{map::sanitize_key, shm::shm_open_read, Shmap};
use env_logger::fmt::Color;
use log::LevelFilter;
use memmap2::Mmap;
use rand::{distributions::Alphanumeric, prelude::SliceRandom, thread_rng, Rng};
use std::io::Write;
use std::{collections::HashSet, str::FromStr, time::Duration};

pub fn init_logger() {
    let level = std::env::var("RUST_LOG").unwrap_or("debug".to_string());
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::from_str(&level).unwrap())
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_bg(Color::Yellow).set_bold(true);

            let timestamp = buf.timestamp();

            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                timestamp,
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or_default(),
                style.value(record.args())
            )
        })
        .try_init();
}

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
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(30);
    let _: String = shmap.get(&key).unwrap().unwrap();
}

#[test]
fn simple_test() {
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(31);
    let value = rand_string(50);

    shmap.insert(&key, value.to_owned()).unwrap();
    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);
    shmap.remove(&key).unwrap();
}

#[test]
fn test_different_size() {
    init_logger();

    let key = rand_string(32);

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
    init_logger();

    let mut secret: Vec<u8> = (0..32).collect();
    secret.shuffle(&mut thread_rng());

    let shmap_enc = Shmap::new_with_encryption(&secret.try_into().unwrap());
    let key = rand_string(33);
    let value = rand_string(50);

    shmap_enc.insert(&key, value.to_owned()).unwrap();
    let ret_value_1: String = shmap_enc.get(&key).unwrap().unwrap();
    assert_eq!(ret_value_1, value);

    // Compare with non-encrypted
    let shmap = Shmap::new();
    let key_2 = rand_string(34);
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
fn test_bad_key() {
    init_logger();

    let key = rand_string(35);
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
    if shmap.get::<String>(&key).is_ok() {
        panic!("It should not have been possible to decrypt here, with a different key")
    }
    shmap.remove(&key).unwrap();
}

#[test]
fn test_set_and_get() {
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(36);
    let value = rand_string(50);

    shmap.insert(&key, value.to_owned()).unwrap();

    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    let ret_value: String = shmap.get(&key).unwrap().unwrap();
    assert_eq!(ret_value, value);

    shmap.remove(&key).unwrap();

    let key = rand_string(37);
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
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(38);
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
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(39);
    let value = rand_string(50);

    shmap.insert(&key, value).unwrap();

    shmap.remove(&key).unwrap();
}

#[test]
fn test_remove_not_found() {
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(40);
    shmap.remove(&key).unwrap();
}

#[test]
#[should_panic(expected = "Option::unwrap()")]
fn test_expiration() {
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(41);
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
    init_logger();

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

    let mut key_to_remove = Vec::new();
    for i in 60..110 {
        let key = rand_string(i);
        shmap.insert(&key, "0").unwrap();
        key_to_remove.push(key);
    }
    key_to_remove.iter().for_each(|key| {
        shmap.remove(key).unwrap();
    });

    fdlimit::raise_fd_limit();
}

// test concurrency between set
#[test]
fn test_set_concurrency() {
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(42);
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
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(43);
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
    init_logger();

    let shmap = Shmap::new();
    let key = rand_string(44);
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
    init_logger();

    let key = rand_string(45);

    let task = move || {
        for i in 0..1024 {
            let shmap = Shmap::new();
            let value = rand_string(i);
            shmap.insert(&key, value.to_owned()).unwrap();
            let _: Option<String> = shmap.get(&key).unwrap();
            shmap.remove(&key).unwrap();
        }
    };

    let mut handles = Vec::new();
    for _i in 0..10 {
        handles.push(std::thread::spawn(task.clone()));
    }
    handles.into_iter().for_each(|t| t.join().unwrap());
}

// test key listing
#[test]
fn test_list_keys() {
    init_logger();

    const NUM: usize = 5;
    let shmap = Shmap::new();

    let keys = (0..NUM).map(rand_string).collect::<HashSet<_>>();
    keys.iter().for_each(|key| {
        let value = rand_string(50);
        shmap.insert(key, value).unwrap();
    });

    // Other tests may run in parallel. Ensure that at least NUM keys are present.
    assert!(shmap.keys().unwrap().len() >= NUM);

    // At least all inserted keys must be present.
    let current_keys = shmap.keys().unwrap().into_iter().collect();
    assert!(keys.is_subset(&current_keys));

    keys.iter().for_each(|key| {
        shmap.remove(key).unwrap();
    });
}
