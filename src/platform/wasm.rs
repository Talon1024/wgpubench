use std::error::Error;
use gloo::net::http::Request;

pub(super) async fn read_text_asset_impl(filename: &str) -> Result<String, Box<dyn Error>> {
    let req_url = String::from("/") + filename;
    let resp = Request::get(&req_url).send().await?;
    Ok(resp.text().await?)
}

pub(super) async fn read_asset_impl(filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let req_url = String::from("/") + filename;
    let resp = Request::get(&req_url).send().await?;
    Ok(resp.binary().await?)
}
