/// Author: Tomasz Kulik
/// 
///

/// The kind of a product that any user can buy or sell in
/// the market.
#[derive(Debug)]
pub enum Product {
    Apple,
    Pear,
    Tomato,
    Potato,
    Onion,
}

impl std::fmt::Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Product::Apple => write!(f, "APPLE"),
            Product::Pear => write!(f, "PEAR"),
            Product::Tomato => write!(f, "TOMATO"),
            Product::Potato => write!(f, "POTATO"),
            Product::Onion => write!(f, "ONION"),
        }
    }
}

/// The structure representing a single transaction
/// between two users in the system. Since the system
/// does not need to indicate the pair of users taking
/// part in the transaction, it is sufficient to
/// keep only the information about the product.
pub struct Transaction(pub Product);
