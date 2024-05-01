use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub electrum: Electrum,
    pub group: GroupConf,
    pub dust: Dust,
    pub fee: Fee,
    pub server: Server,
    pub network: Network,
}

#[derive(Deserialize)]
pub struct Electrum {
    pub endpoint: String,
    pub certificate_validation: bool,
}

#[derive(Deserialize)]
pub struct GroupConf {
    pub max_time: usize,
    pub max_size: usize,
}

#[derive(Deserialize)]
pub struct Dust {
    pub limit: u64,
}

#[derive(Deserialize)]
pub struct Fee {
    pub range: f32,
}

#[derive(Deserialize)]
pub struct Server {
    pub ip: String,
    pub port: String,
}

#[derive(Deserialize)]
pub struct Network {
    pub name: String,
}
