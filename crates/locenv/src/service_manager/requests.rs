use reqmap_macros::HttpRequest;

#[derive(HttpRequest)]
pub enum Request {
    #[get("/status")]
    GetStatus,
}
