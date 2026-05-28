use asiri_core::{split_secret, recover_secret, Share};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Serialize, Deserialize)]
pub struct ShareDto {
    pub index: u8,
    pub data: String, // Hex encoded
}

#[tauri::command]
fn split_secret_cmd(mut secret: String, threshold: u8, shares: u8) -> Result<Vec<ShareDto>, String> {
    let result = match split_secret(secret.as_bytes(), threshold, shares) {
        Ok(generated_shares) => {
            let dtos = generated_shares.into_iter().map(|s| ShareDto {
                index: s.index,
                data: hex::encode(&s.data),
            }).collect();
            Ok(dtos)
        }
        Err(e) => Err(e.to_string()),
    };
    secret.zeroize();
    result
}

#[tauri::command]
fn recover_secret_cmd(shares: Vec<ShareDto>) -> Result<String, String> {
    let mut core_shares = Vec::new();
    for dto in shares {
        let data = hex::decode(&dto.data).map_err(|e| format!("Invalid hex: {}", e))?;
        core_shares.push(Share { index: dto.index, data });
    }

    match recover_secret(&core_shares) {
        Ok(secret_bytes) => {
            // Try to parse as UTF-8 string
            match String::from_utf8(secret_bytes.to_vec()) {
                Ok(s) => Ok(s),
                Err(_) => Ok(hex::encode(&*secret_bytes)), // Fallback to hex
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            split_secret_cmd,
            recover_secret_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
