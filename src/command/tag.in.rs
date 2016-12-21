#[derive(Clone, Debug, Deserialize, Serialize)]
struct Tag {
    name: String,
    content: String,
    // TODO: change this to `UserId`.
    owner_id: u64,
    uses: u32,
    // TODO: change this to `GuildId`.
    location: Option<String>,
    created_at: DateTime<UTC>,
}
