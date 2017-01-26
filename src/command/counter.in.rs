#[derive(Clone, Debug, Deserialize, Serialize)]
struct Counter {
    name: String,
    count: i64,
    owner_id: u64,
    // Whether or not the counter is publically-editable.
    public_edit: bool,
    queries: u32,
    location: Option<String>,
    created_at: DateTime<UTC>,
    blacklisted_users: HashSet<u64>,
    whitelisted_users: HashSet<u64>,
}
