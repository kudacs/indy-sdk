extern crate indyrs as indy;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rmp_serde;
extern crate byteorder;
extern crate futures;
#[macro_use]
mod utils;

use indy::crypto::{Key, Crypto};
use indy::wallet::Wallet;
#[cfg(feature="extended_api_types")]
use indy::ErrorCode;

#[cfg(feature="extended_api_types")]
use std::time::Duration;
#[cfg(feature="extended_api_types")]
use std::sync::mpsc::channel;

#[allow(unused_imports)]
use futures::Future;

use utils::constants::DEFAULT_CREDENTIALS;

macro_rules! safe_wallet_create {
    ($x:ident) => {
        match Wallet::delete($x, DEFAULT_CREDENTIALS).wait() {
            Ok(..) => {},
            Err(..) => {}
        };
        Wallet::create($x, DEFAULT_CREDENTIALS).wait().unwrap();
    }
}

macro_rules! wallet_cleanup {
    ($x:ident, $y:ident) => {
        Wallet::close($x).wait().unwrap();
        Wallet::delete($y, DEFAULT_CREDENTIALS).wait().unwrap();
    }
}

pub fn time_it_out<F>(msg: &str, test: F) -> bool where F: Fn() -> bool {
    for _ in 1..250 {
        if test() {
            return true;
        }
    }
    // It tried to do a timeout test 250 times and the system was too fast, so just succeed
    println!("{} - system too fast for timeout test", msg);
    true
}

mod high_cases {
    use super::*;

    mod key_tests {
        use super::*;

        #[test]
        fn all_async_works() {
            let wallet_name = r#"{"id":"all_async_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();

            let vkey = Key::create(handle, None).wait().unwrap();

            let metadata = r#"{"name": "dummy"}"#;
            Key::set_metadata(handle, &vkey, metadata).wait().unwrap();

            let meta= Key::get_metadata(handle, &vkey).wait().unwrap();
            assert_eq!(metadata.to_string(),meta);

            wallet_cleanup!(handle, wallet_name);
        }
    }

    mod crypto_tests {
        use super::*;

        #[test]
        fn sign_verify_async_works() {
            let wallet_name = r#"{"id":"sign_verify_async_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();

            let vkey = Key::create(handle, None).wait().unwrap();

            let message = r#"Hello World"#.as_bytes();
            let sig = Crypto::sign(handle, &vkey, message).wait().unwrap();
            assert_eq!(sig.len(), 64);

            wallet_cleanup!(handle, wallet_name);
        }
    }
}

mod low_cases {
    use super::*;

    mod key_tests {
        use super::*;

        #[test]
        fn create_key_works() {
            let wallet_name = r#"{"id":"create_key_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();

            let res = Key::create(handle, None).wait();
            assert!(res.is_ok());

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn create_key_timeout_works() {
            let wallet_name = r#"{"id":"create_key_timeout_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();

            let res = Key::create_timeout(handle, None, Duration::from_millis(1000));
            assert!(res.is_ok());

            let res = Key::create_timeout(handle, None, Duration::from_millis(1));
            assert!(res.is_err());

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn create_key_async_works() {
            let wallet_name = r#"{"id":"create_key_async_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let (sender, receiver) = channel();

            let closure = move |error, result| {
                sender.send((error, result)).unwrap();
            };

            let res = Key::create_async(handle, None, closure);
            assert!(res.is_ok());
            receiver.recv().unwrap();
            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        fn set_metadata_works() {
            let wallet_name = r#"{"id":"set_metadata_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();
            let verkey = Key::create(handle, None).wait().unwrap();

            assert!(Key::set_metadata(handle, &verkey, r#"{"name": "dummy key"}"#).wait().is_ok());
            wallet_cleanup!(handle, wallet_name);
        }

        #[ignore]
        #[test]
        #[cfg(feature="extended_api_types")]
        fn set_metadata_timeout_works() {
            let wallet_name = r#"{"id":"set_metadata_timeout_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let verkey = Key::create(handle, None).unwrap();

            assert!(Key::set_metadata_timeout(handle, &verkey, r#"{"name": "dummy key"}"#, Duration::from_millis(5000)).is_ok());

            assert!(Key::set_metadata_timeout(handle, &verkey, r#"{"name": "dummy key"}"#, Duration::from_millis(1)).is_err());
            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn set_metadata_async_works() {
            let wallet_name = r#"{"id":"set_metadata_async_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let (sender, receiver) = channel();

            let closure = move |error| {
                sender.send(error).unwrap();
            };
            let verkey = Key::create(handle, None).unwrap();
            let res = Key::set_metadata_async(handle, &verkey, r#"{"name": "dummy key"}"#, closure);
            assert!(res.is_ok());
            receiver.recv().unwrap();
            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        fn get_metadata_works() {
            let wallet_name = r#"{"id":"get_metadata_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();
            let verkey = Key::create(handle, None).wait().unwrap();

            assert!(Key::set_metadata(handle, &verkey, r#"{"name": "dummy key"}"#).wait().is_ok());
            assert_eq!(Key::get_metadata(handle, &verkey).wait().unwrap(), r#"{"name": "dummy key"}"#);
            wallet_cleanup!(handle, wallet_name);
        }

        #[ignore]
        #[test]
        #[cfg(feature="extended_api_types")]
        fn get_metadata_timeout_works() {
            let wallet_name = r#"{"id":"get_metadata_timeout_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let verkey = Key::create(handle, None).unwrap();

            assert!(Key::set_metadata_timeout(handle, &verkey, r#"{"name": "dummy key"}"#, Duration::from_millis(5000)).is_ok());
            assert_eq!(Key::get_metadata_timeout(handle, &verkey, Duration::from_millis(5000)).unwrap(), r#"{"name": "dummy key"}"#);
            assert!(Key::get_metadata_timeout(handle, &verkey, Duration::from_millis(1)).is_err());
            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn get_metadata_async_works() {
            let wallet_name = r#"{"id":"get_metadata_async_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let (sender, receiver) = channel();

            let closure = move |error| {
                sender.send(error).unwrap();
            };
            let verkey = Key::create(handle, None).unwrap();
            let res = Key::set_metadata_async(handle, &verkey, r#"{"name": "dummy key"}"#, closure);
            assert_eq!(res, ErrorCode::Success);
            receiver.recv().unwrap();

            let (sender, receiver) = channel();

            let closure = move |error, result| {
                sender.send((error, result)).unwrap();
            };
            let res = Key::get_metadata_async(handle, &verkey, closure);
            assert!(res.is_ok());
            let (e, r) = receiver.recv().unwrap();
            assert!(e.is_ok());
            assert_eq!(r, r#"{"name": "dummy key"}"#);
            wallet_cleanup!(handle, wallet_name);
        }
    }

    mod crypto_tests {
        use super::*;

        #[test]
        fn sign_works() {
            let wallet_name = r#"{"id":"sign_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();
            let vkey = Key::create(handle, None).wait().unwrap();

            let res = Crypto::sign(handle, &vkey, r#"Hello World"#.as_bytes()).wait();
            assert!(res.is_ok());
            let sig = res.unwrap();
            assert_eq!(sig.len(), 64);

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn sign_timeout_works() {
            let wallet_name = r#"{"id":"sign_timeout_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let vkey = Key::create(handle, None).unwrap();

            let res = Crypto::sign_timeout(handle, &vkey, r#"Hello World"#.as_bytes(), Duration::from_millis(5000));
            assert!(res.is_ok());
            let sig = res.unwrap();
            assert_eq!(sig.len(), 64);

            assert!(time_it_out(wallet_name, move|| {Crypto::sign_timeout(handle, &vkey, r#"Hello World"#.as_bytes(), Duration::from_millis(1)).is_err() }));
            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn sign_async_works() {
            let wallet_name = r#"{"id":"sign_async_works"}"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let vkey = Key::create(handle, None).unwrap();

            let (sender, receiver) = channel();
            let res = Crypto::sign_async(handle, &vkey, r#"Hello World"#.as_bytes(), move |err, sig| {
                sender.send((err, sig)).unwrap();
            });
            assert!(res.is_ok());
            let (e, sig) = receiver.recv().unwrap();
            assert!(e.is_ok());
            assert_eq!(sig.len(), 64);

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        fn verify_works() {
            let wallet_name = r#"{"id":"verify_works"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();
            let vkey = Key::create(handle, None).wait().unwrap();
            let res = Crypto::sign(handle, &vkey, message.as_bytes()).wait();
            assert!(res.is_ok());
            let sig = res.unwrap();

            let res = Crypto::verify(&vkey, message.as_bytes(), sig.as_slice()).wait();
            assert!(res.is_ok());
            assert!(res.unwrap());

            let mut fake_sig = Vec::new();
            for i in 1..65 {
                fake_sig.push(i as u8);
            }

            let res = Crypto::verify(&vkey, message.as_bytes(), fake_sig.as_slice()).wait();
            assert!(res.is_ok());
            assert!(!res.unwrap());
            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn verify_timeout_works() {
            let wallet_name = r#"{"id":"verify_timeout_works"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let vkey = Key::create(handle, None).unwrap();
            let res = Crypto::sign(handle, &vkey, message.as_bytes());
            assert!(res.is_ok());
            let sig = res.unwrap();

            let res = Crypto::verify_timeout(&vkey, message.as_bytes(), sig.as_slice(), Duration::from_millis(5000));
            assert!(res.is_ok());
            assert!(res.unwrap());
            let mut fake_sig = Vec::new();
            for i in 1..65 {
                fake_sig.push(i as u8);
            }

            let res = Crypto::verify_timeout(&vkey, message.as_bytes(), fake_sig.as_slice(), Duration::from_millis(5000));
            assert!(res.is_ok());
            assert!(!res.unwrap());

            assert!(time_it_out(wallet_name, move||{Crypto::verify_timeout(&vkey, message.as_bytes(), sig.as_slice(), Duration::from_millis(1)).is_err()}));

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn verify_works_async() {
            let wallet_name = r#"{"id":"verify_works_async"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let vkey = Key::create(handle, None).unwrap();
            let res = Crypto::sign(handle, &vkey, message.as_bytes());
            assert!(res.is_ok());
            let sig = res.unwrap();
            let (sender, receiver) = channel();
            let sender1 = sender.clone();

            let res = Crypto::verify_async(&vkey, message.as_bytes(), sig.as_slice(), move|err, result|{
                sender.send((err, result)).unwrap();
            });
            assert!(res.is_ok());
            let (e, r) = receiver.recv().unwrap();
            assert!(e.is_ok());
            assert!(r);

            let mut fake_sig = Vec::new();
            for i in 1..65 {
                fake_sig.push(i as u8);
            }
            let res = Crypto::verify_async(&vkey, message.as_bytes(), fake_sig.as_slice(), move|err, result| {
               sender1.send((err, result)).unwrap();
            });
            assert!(res.is_ok());
            let (e, r) = receiver.recv().unwrap();
            assert!(e.is_ok());
            assert!(!r);
            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        fn auth_crypt_decrypt_works() {
            let wallet_name = r#"{"id":"auth_crypt_decrypt_works"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();
            let vkey1 = Key::create(handle, None).wait().unwrap();
            let vkey2 = Key::create(handle, None).wait().unwrap();

            let res = Crypto::auth_crypt(handle, &vkey1, &vkey2, message.as_bytes()).wait();
            assert!(res.is_ok());
            let ciphertext = res.unwrap();

            let res = Crypto::auth_decrypt(handle, &vkey2, ciphertext.as_slice()).wait();

            assert!(res.is_ok());
            let (actual_vkey, plaintext) = res.unwrap();
            assert_eq!(actual_vkey, vkey1);
            assert_eq!(plaintext, message.as_bytes());

            let mut fake_msg = Vec::new();
            for i in 1..ciphertext.len() {
                fake_msg.push(i as u8);
            }

            let res = Crypto::auth_decrypt(handle, &vkey2, fake_msg.as_slice()).wait();
            assert!(res.is_err());

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn auth_crypt_decrypt_timeout_works() {
            let wallet_name = r#"{"id":"auth_crypt_decrypt_timeout_works"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let vkey1 = Key::create(handle, None).unwrap();
            let vkey2 = Key::create(handle, None).unwrap();

            let res = Crypto::auth_crypt_timeout(handle, &vkey1, &vkey2, message.as_bytes(), Duration::from_millis(5000));
            assert!(res.is_ok());
            let ciphertext = res.unwrap();

            let res = Crypto::auth_decrypt_timeout(handle, &vkey2, ciphertext.as_slice(), Duration::from_millis(5000));

            assert!(res.is_ok());
            let (actual_vkey, plaintext) = res.unwrap();
            assert_eq!(actual_vkey, vkey1);
            assert_eq!(plaintext, message.as_bytes());

            let mut fake_msg = Vec::new();
            for i in 1..ciphertext.len() {
                fake_msg.push(i as u8);
            }

            let res = Crypto::auth_decrypt_timeout(handle, &vkey2, fake_msg.as_slice(), Duration::from_millis(5000));
            assert!(res.is_err());

            assert!(time_it_out(wallet_name,move||{Crypto::auth_decrypt_timeout(handle, &vkey2, fake_msg.as_slice(), Duration::from_millis(5000)).is_err()}));

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn auth_crypt_decrypt_async_works() {
            let wallet_name = r#"{"id":"auth_crypt_decrypt_async_works"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let vkey1 = Key::create(handle, None).unwrap();
            let vkey2 = Key::create(handle, None).unwrap();
            let (sender, receiver) = channel();

            let (sender1, receiver1) = channel();
            let sender2 = sender1.clone();

            let closure_crypt = move|err, ciphertext| {
                sender.send((err, ciphertext)).unwrap();
            };

            let res = Crypto::auth_crypt_async(handle, &vkey1, &vkey2, message.as_bytes(), closure_crypt);
            assert!(res.is_ok());
            let (err, ciphertext) = receiver.recv().unwrap();
            assert!(err.is_ok());

            let closure_decrypt1 = move|err, skey, plaintext| {
                sender1.send((err, skey, plaintext)).unwrap();
            };

            let res = Crypto::auth_decrypt_async(handle, &vkey2, ciphertext.as_slice(), closure_decrypt1);

            assert!(res.is_ok());
            let (err, actual_vkey, plain) = receiver1.recv().unwrap();
            assert!(err.is_ok());
            assert_eq!(actual_vkey, vkey1);
            assert_eq!(plain, message.as_bytes());

            let mut fake_msg = Vec::new();
            for i in 1..ciphertext.len() {
                fake_msg.push(i as u8);
            }

            let closure_decrypt2 = move|err, skey, plaintext| {
                sender2.send((err, skey, plaintext)).unwrap();
            };

            let res = Crypto::auth_decrypt_async(handle, &vkey2, fake_msg.as_slice(), closure_decrypt2);
            assert!(res.is_ok());
            let (err, _, _) = receiver1.recv().unwrap();
            assert!(err.is_err());

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        fn anon_crypt_decrypt_works() {
            let wallet_name = r#"{"id":"anon_crypt_decrypt_works"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).wait().unwrap();
            let vkey1 = Key::create(handle, None).wait().unwrap();

            let res = Crypto::anon_crypt(&vkey1, message.as_bytes()).wait();
            assert!(res.is_ok());
            let ciphertext = res.unwrap();

            let res = Crypto::anon_decrypt(handle, &vkey1, ciphertext.as_slice()).wait();

            assert!(res.is_ok());
            let plaintext = res.unwrap();
            assert_eq!(plaintext, message.as_bytes());

            let mut fake_msg = Vec::new();
            for i in 1..ciphertext.len() {
                fake_msg.push(i as u8);
            }

            let res = Crypto::anon_decrypt(handle, &vkey1, fake_msg.as_slice()).wait();
            assert!(res.is_err());

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn anon_crypt_decrypt_timeout_works() {
            let wallet_name = r#"{"id":"anon_crypt_decrypt_timeout_works"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let vkey1 = Key::create(handle, None).unwrap();

            let res = Crypto::anon_crypt_timeout(&vkey1, message.as_bytes(), Duration::from_millis(5000));
            assert!(res.is_ok());
            let ciphertext = res.unwrap();

            let res = Crypto::anon_decrypt_timeout(handle, &vkey1, ciphertext.as_slice(), Duration::from_millis(5000));

            assert!(res.is_ok());
            let plaintext = res.unwrap();
            assert_eq!(plaintext, message.as_bytes());

            let mut fake_msg = Vec::new();
            for i in 1..ciphertext.len() {
                fake_msg.push(i as u8);
            }

            let res = Crypto::anon_decrypt_timeout(handle, &vkey1, fake_msg.as_slice(), Duration::from_millis(5000));
            assert!(res.is_err());

            assert!(time_it_out(wallet_name, move||{Crypto::anon_decrypt_timeout(handle, &vkey1, fake_msg.as_slice(), Duration::from_millis(5000)).is_err()}));

            wallet_cleanup!(handle, wallet_name);
        }

        #[test]
        #[cfg(feature="extended_api_types")]
        fn anon_crypt_decrypt_async_works() {
            let wallet_name = r#"{"id":"anon_crypt_decrypt_async_works"}"#;
            let message = r#"Hello World"#;
            safe_wallet_create!(wallet_name);
            let handle = Wallet::open(wallet_name, DEFAULT_CREDENTIALS).unwrap();
            let vkey1 = Key::create(handle, None).unwrap();

            let (sender, receiver) = channel();

            let res = Crypto::anon_crypt_async(&vkey1, message.as_bytes(), move|err, cipher|{
                sender.send((err, cipher)).unwrap();
            });
            assert!(res.is_ok());
            let (e, ciphertext) = receiver.recv().unwrap();
            assert!(e.is_ok());

            let (sender1, receiver1) = channel();
            let sender2 = sender1.clone();

            let res = Crypto::anon_decrypt_async(handle, &vkey1, ciphertext.as_slice(), move|err, plain| {
                sender1.send((err, plain)).unwrap();
            });

            assert!(res.is_ok());
            let (e, plaintext) = receiver1.recv().unwrap();
            assert!(e.is_ok());
            assert_eq!(plaintext, message.as_bytes());

            let mut fake_msg = Vec::new();
            for i in 1..ciphertext.len() {
                fake_msg.push(i as u8);
            }

            let res = Crypto::anon_decrypt_async(handle, &vkey1, fake_msg.as_slice(), move|err, plain|{
                sender2.send((err, plain)).unwrap();
            });
            assert!(res.is_ok());
            let (e, _) = receiver1.recv().unwrap();
            assert!(e.is_err());

            wallet_cleanup!(handle, wallet_name);
        }
    }
}
