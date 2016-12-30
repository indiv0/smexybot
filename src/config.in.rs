#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Name by which the bot is referred to internally (e.g. in debug output).
    pub bot_name: String,
    /// The command prefix for the bot.
    pub command_prefix: String,
    /// The URL for the source of the project.
    pub source_url: String,
    /// The authors to use in author checks for permissions.
    pub owners: HashSet<u64>,
}
