use serde::{Deserialize, Serialize};
use std::env::current_dir;
use tableland_client::ChainID;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub chain: Chain,
    pub cache: CacheConfig,
    pub ipfs: IpfsConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chain {
    pub id: ChainID,
}

impl Default for Config {
    fn default() -> Self {
        let dir = current_dir().expect("unable to get current directory");
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: "3030".to_string(),
            },
            chain: Chain { id: ChainID::Local },
            cache: CacheConfig {
                directory: dir.to_str().unwrap().to_string(),
            },
            ipfs: IpfsConfig {
                gateway: "http://localhost:8081/ipfs".to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheConfig {
    pub directory: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpfsConfig {
    pub gateway: String,
}
