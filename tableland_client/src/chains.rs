const MAINNET_URL: &str = "https://tableland.network";
const TESTNET_URL: &str = "https://testnets.tableland.network";
const LOCAL_URL: &str = "http://localhost:8080";

#[allow(dead_code)]
pub enum ChainID {
    Ethereum = 1,
    Optimism = 10,
    Polygon = 137,
    Arbitrum = 42161,
    EthereumGoerli = 5,
    OptimismGoerli = 420,
    ArbitrumGoerli = 421613,
    PolygonMumbai = 80001,
    Local = 31337,
}

pub struct Chain {
    pub endpoint: String,
    pub id: i32,
    pub name: String,
}

impl Chain {
    fn can_relay_writes(&self) -> bool {
        &self.endpoint != MAINNET_URL
    }
}

pub fn get_chain(id: ChainID) -> Chain {
    match id {
        ChainID::Ethereum => Chain {
            endpoint: MAINNET_URL.to_string(),
            id: id as i32,
            name: "Ethereum".to_string(),
        },
        ChainID::Optimism => Chain {
            endpoint: MAINNET_URL.to_string(),
            id: id as i32,
            name: "Optimism".to_string(),
        },
        ChainID::Polygon => Chain {
            endpoint: MAINNET_URL.to_string(),
            id: id as i32,
            name: "Polygon".to_string(),
        },
        ChainID::Arbitrum => Chain {
            endpoint: MAINNET_URL.to_string(),
            id: id as i32,
            name: "Arbitrum".to_string(),
        },
        ChainID::EthereumGoerli => Chain {
            endpoint: TESTNET_URL.to_string(),
            id: id as i32,
            name: "EthereumGoerli".to_string(),
        },
        ChainID::OptimismGoerli => Chain {
            endpoint: TESTNET_URL.to_string(),
            id: id as i32,
            name: "OptimismGoerli".to_string(),
        },
        ChainID::ArbitrumGoerli => Chain {
            endpoint: TESTNET_URL.to_string(),
            id: id as i32,
            name: "ArbitrumGoerli".to_string(),
        },
        ChainID::PolygonMumbai => Chain {
            endpoint: TESTNET_URL.to_string(),
            id: id as i32,
            name: "PolygonMumbai".to_string(),
        },
        ChainID::Local => Chain {
            endpoint: LOCAL_URL.to_string(),
            id: id as i32,
            name: "Local".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_chain_works() {
        let chain = get_chain(ChainID::Ethereum);
        assert_eq!(chain.endpoint, MAINNET_URL);
        assert_eq!(chain.id, 1);
        assert_eq!(chain.name, "Ethereum");
    }

    #[test]
    fn can_relay_writes_works() {
        let chain = get_chain(ChainID::Ethereum);
        let result = chain.can_relay_writes();
        assert_eq!(result, false);
    }
}
