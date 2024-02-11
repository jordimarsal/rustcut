use rand::Rng;
use std::iter;
use rand::distributions::Alphanumeric;

pub fn create_api_key() -> String {
    let mut rng = rand::thread_rng();
    let api_key: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric) as char) // Convert u8 to char
        .take(32)
        .collect();

    api_key
}
