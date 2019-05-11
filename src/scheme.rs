table! {
    /// The table containing all user specific settings.
    user_settings (id) {
        /// The ID of the user stored in this row.
        id -> BigInt,
        /// Whether the user is blacklisted from using the bot entirely.
        blacklisted -> Bool,
    }
}

table! {
    /// The table containing all server specific settings.
    server_settings (id) {
        /// The ID of the server stored in this row.
        id -> BigInt,
        /// Whether the server is blacklisted from using the bot entirely.
        blacklisted -> Bool,
    }
}
