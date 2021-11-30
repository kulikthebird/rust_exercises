/// Author: Tomasz Kulik
/// 
///

use crate::transaction::Product;

/// The unique User ID
///
pub type UserId = u16;

/// A convinient type representing a user single order
///
#[derive(Debug)]
pub enum Order {
    Buy(UserId, Product),
    Sell(UserId, Product),
}

impl Order {
    pub fn new_order_form_str(user_id: UserId, input: &str) -> Result<Order, String> {
        match input {
            "BUY:APPLE" => Ok(Order::Buy(user_id, Product::Apple)),
            "BUY:PEAR" => Ok(Order::Buy(user_id, Product::Pear)),
            "BUY:TOMATO" => Ok(Order::Buy(user_id, Product::Tomato)),
            "BUY:POTATO" => Ok(Order::Buy(user_id, Product::Potato)),
            "BUY:ONION" => Ok(Order::Buy(user_id, Product::Onion)),
            "SELL:APPLE" => Ok(Order::Sell(user_id, Product::Apple)),
            "SELL:PEAR" => Ok(Order::Sell(user_id, Product::Pear)),
            "SELL:TOMATO" => Ok(Order::Sell(user_id, Product::Tomato)),
            "SELL:POTATO" => Ok(Order::Sell(user_id, Product::Potato)),
            "SELL:ONION" => Ok(Order::Sell(user_id, Product::Onion)),
            _ => Err(format!("Unknown order: {}", input)),
        }
    }
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::Buy(user_id, product) => write!(f, "new buy order ({}, {})", user_id, product),
            Order::Sell(user_id, product) => write!(f, "new sell order ({}, {})", user_id, product),
        }
    }
}
