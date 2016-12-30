#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// The command prefix for the bot.
    pub command_prefix: String,
    /// The URL for the source of the project.
    pub source_url: String,
    /// The authors to use in author checks for permissions.
    pub owners: HashSet<u64>,
}
