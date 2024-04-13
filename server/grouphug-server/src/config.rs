// Electrum Server endpoints
/// Mainnet
//pub const MAINNET_ELECTRUM_SERVER_ENDPOINT: &str = "ssl://electrum.blockstream.info:50001";
/// Testnet  
pub const TESTNET_ELECTRUM_SERVER_ENDPOINT: &str = "ssl://electrum.blockstream.info:60002";

/// Time to wait until closing a group if it is not fulfilled (in seconds).
//5min -> 300 | 12h -> 43200
//pub const MAX_TIME: usize = 300;

/// Maximum number of participants of each group.
pub const MAX_SIZE: usize = 3;


/// Dust limit in sats
pub const DUST_LIMIT: u64 = 1000;


/// Fee range
pub const FEE_RANGE: f32 = 2.0;