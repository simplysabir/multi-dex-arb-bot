use std::error::Error;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::Rng;
use chrono::Utc;
use log::{info, warn, error};
use env_logger;

// Simulated DEX API client
struct DexClient {
    name: String,
}

impl DexClient {
    fn new(name: &str) -> Self {
        DexClient {
            name: name.to_string(),
        }
    }

    async fn get_price(&self, pair: &str) -> Result<f64, Box<dyn Error>> {
        // Simulate API call delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Generate a random price for simulation
        let base_price = 1000.0; // Example base price for ETH/USDC
        let variation = rand::thread_rng().gen_range(-5.0..5.0);
        Ok(base_price + variation)
    }

    async fn execute_trade(&self, pair: &str, amount: f64) -> Result<(), Box<dyn Error>> {
        // Simulate trade execution
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        info!("Executed trade on {}: {} {}", self.name, amount, pair);
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct PriceData {
    exchange: String,
    pair: String,
    price: f64,
    timestamp: chrono::DateTime<Utc>,
}

struct ArbitrageBot {
    dex_clients: HashMap<String, DexClient>,
    price_history: Arc<Mutex<Vec<PriceData>>>,
}

impl ArbitrageBot {
    fn new() -> Self {
        let mut dex_clients = HashMap::new();
        dex_clients.insert("DEX1".to_string(), DexClient::new("DEX1"));
        dex_clients.insert("DEX2".to_string(), DexClient::new("DEX2"));
        dex_clients.insert("DEX3".to_string(), DexClient::new("DEX3"));

        ArbitrageBot {
            dex_clients,
            price_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn monitor_prices(&self, pair: &str) -> Result<Vec<PriceData>, Box<dyn Error>> {
        let mut prices = vec![];

        for (name, client) in &self.dex_clients {
            match client.get_price(pair).await {
                Ok(price) => {
                    let price_data = PriceData {
                        exchange: name.clone(),
                        pair: pair.to_string(),
                        price,
                        timestamp: Utc::now(),
                    };
                    prices.push(price_data.clone());

                    // Update price history
                    self.price_history.lock().await.push(price_data);
                }
                Err(e) => {
                    warn!("Failed to get price from {}: {}", name, e);
                }
            }
        }

        Ok(prices)
    }

    async fn check_arbitrage_opportunity(&self, prices: &[PriceData]) -> Option<(String, String, f64)> {
        if prices.len() < 2 {
            return None;
        }

        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;
        let mut min_exchange = String::new();
        let mut max_exchange = String::new();

        for price_data in prices {
            if price_data.price < min_price {
                min_price = price_data.price;
                min_exchange = price_data.exchange.clone();
            }
            if price_data.price > max_price {
                max_price = price_data.price;
                max_exchange = price_data.exchange.clone();
            }
        }

        let profit_margin = (max_price - min_price) / min_price;
        
        // Assuming a 0.5% threshold for profit after fees
        if profit_margin > 0.005 {
            Some((min_exchange, max_exchange, min_price))
        } else {
            None
        }
    }

    async fn execute_trade(&self, from_dex: &str, to_dex: &str, amount: f64) -> Result<(), Box<dyn Error>> {
        let from_client = self.dex_clients.get(from_dex).ok_or("From DEX not found")?;
        let to_client = self.dex_clients.get(to_dex).ok_or("To DEX not found")?;

        // Execute buy on the cheaper exchange
        from_client.execute_trade("ETH/USDC", amount).await?;

        // Execute sell on the more expensive exchange
        to_client.execute_trade("ETH/USDC", -amount).await?;

        info!("Arbitrage executed: Bought {} ETH on {}, Sold on {}", amount, from_dex, to_dex);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let bot = ArbitrageBot::new();
    let pair = "ETH/USDC";

    info!("Starting arbitrage bot for pair: {}", pair);

    loop {
        let prices = bot.monitor_prices(pair).await?;
        
        if let Some((from_dex, to_dex, amount)) = bot.check_arbitrage_opportunity(&prices).await {
            match bot.execute_trade(&from_dex, &to_dex, amount).await {
                Ok(()) => info!("Arbitrage trade executed successfully"),
                Err(e) => error!("Failed to execute arbitrage trade: {}", e),
            }
        }
        
        // Add delay to control trading frequency
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}