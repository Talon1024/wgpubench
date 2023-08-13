use std::{
    fs::File,
    io::{Read, Result},
};

pub(super) async fn read_text_asset_impl(filename: &str) -> Result<String> {
    let mut file = File::open(filename)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

pub(super) async fn read_asset_impl(filename: &str) -> Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}
