#[derive(Debug, Deserialize)]
struct CseResponse {
    pub items: Vec<CseItem>,
}

#[derive(Debug, Deserialize)]
struct CseItem {
    pub link: String,
}
