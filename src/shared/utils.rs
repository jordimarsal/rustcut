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

pub fn create_random_key(length: usize) -> String {
    let key1 = generate_key_part(length);
    let key2 = generate_key_part(length);
    format!("{}_{}", key1, key2)
}

fn generate_key_part(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_api_key_length_and_unique() {
        let a = create_api_key();
        let b = create_api_key();
        assert_eq!(a.len(), 32);
        assert_eq!(b.len(), 32);
        assert_ne!(a, b);
    }

    #[test]
    fn create_random_key_parts_and_length() {
        let key = create_random_key(8);
        let parts: Vec<&str> = key.split('_').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 8);
    }
}
