use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::env;
use log::error;

pub fn encrypt_token(token_to_encrypt: &str) -> Result<String, String> {


    // Load .env variables (`env::var` will read ~.config/absotui/.env)
    // check `main.rs` to see the init process for dotenv
    // Retrieve secret key from .env
    let _secret_key = match env::var("ABSOTUI_SECRET_KEY") {
        Ok(key) => {

            // Create magic crypt object
            let mc = new_magic_crypt!(key, 256);

            // Token encryption
            let encrypted_token = mc.encrypt_str_to_base64(token_to_encrypt);

            return Ok(encrypted_token)
        }
        Err(_) => {
            error!("No secret found in .env. Do this:\n
                mkdir -p ~/.config/absotui\n
                echo 'ABSOTUI_SECRET_KEY=secret' >> ~/.config/absotui/.env");
            return Err("No secret found in .env. Do this:\n
                mkdir -p ~/.config/absotui\n
                echo 'ABSOTUI_SECRET_KEY=secret' >> ~/.config/absotui/.env".to_string()); 
        },
    };
}


pub fn decrypt_token(encrypted_token: &str) -> Result<String, String> {
    // Load .env variables (`env::var` will read ~.config/absotui/.env)
    // check `main.rs` to see the init process for dotenv
    // Retrieve secret key from .env
    let secret_key = match env::var("ABSOTUI_SECRET_KEY") {
        Ok(key) => {
            // Create magic crypt object
            let mc = new_magic_crypt!(key, 256);

            // Token decryption
            match mc.decrypt_base64_to_string(encrypted_token) {
                Ok(decrypted_token) => Ok(decrypted_token), 
                Err(_) => {
                    error!("Failed to decrypt the token.");
                    Err("Failed to decrypt the token.".to_string())
                }
            }
        }
        Err(_) => {
            error!("No secret found in .env. Do this:\n
                mkdir -p ~/.config/absotui\n
                echo 'ABSOTUI_SECRET_KEY=secret' >> ~/.config/absotui/.env");
            Err("No secret found in .env. Do this:\n
                mkdir -p ~/.config/absotui\n
                echo 'ABSOTUI_SECRET_KEY=secret' >> ~/.config/absotui/.env".to_string()) 
        },
    };

    secret_key
}
