#[cfg(not(target_family="wasm"))]
mod native;
#[cfg(not(target_family="wasm"))]
use native::*;

#[cfg(target_family="wasm")]
mod wasm;
#[cfg(target_family="wasm")]
use wasm::*;

use std::error::Error;

// 'static ensures the string is built-in
pub async fn read_text_asset(filename: &'static str) -> Result<String, Box<dyn Error>> {
    read_text_asset_impl(filename).await.map_err(Box::from)
}

pub async fn read_asset(filename: &'static str) -> Result<Vec<u8>, Box<dyn Error>> {
    read_asset_impl(filename).await.map_err(Box::from)
}
