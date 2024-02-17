use rand::distributions::Alphanumeric;
use rand::Rng;
use std::iter;

pub fn create_api_key() -> String {
    let mut rng = rand::thread_rng();
    let api_key: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric) as char) // Convert u8 to char
        .take(32)
        .collect();

    api_key
}

pub fn create_unique_random_key() -> String {
    let mut rng = rand::thread_rng();
    let key: u64 = rng.gen();
    key.to_string()
}

pub fn create_random_key(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let key: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric) as char) // Convert u8 to char
        .take(length)
        .collect();
    key
}
