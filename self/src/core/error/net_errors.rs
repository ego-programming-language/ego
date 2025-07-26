pub enum NetErrors {
    // in the future we should have more granularity about why couldnt connect
    NetConnectError(String),
}
