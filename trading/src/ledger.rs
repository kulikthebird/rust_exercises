/// Author: Tomasz Kulik
/// 
///

use crate::order::Order;
///
/// This module implements the bussiness logic of the system.
///
use crate::transaction::{Product, Transaction};

/// A Ledger of a given Product.
///
/// Since the system does not track the authors
/// of the orders, it's sufficient to keep only the
/// current balance of the given product.
/// Positive value means that there's more people
/// wanting to sell some product, accordingly negative
/// number indicates people waiting to buy the product.
pub type ProductLedger = i64;

/// This is a structure containing all the ledgers that
/// are present in the trading market.
pub struct Ledger {
    apples: ProductLedger,
    pears: ProductLedger,
    tomatoes: ProductLedger,
    potatoes: ProductLedger,
    onions: ProductLedger,
}

/// Main ledger in the system.
///
/// It stores the current state of the market, i.e.
/// ledgers of a given products. It also keeps the
/// state up-to-date by updating the ledgers with the
/// users' orders. As a result it generates transactions
/// if any pair of orders matches.
impl Ledger {
    /// Create a new empty Ledger.
    pub fn new() -> Ledger {
        Ledger {
            apples: 0,
            pears: 0,
            tomatoes: 0,
            potatoes: 0,
            onions: 0,
        }
    }

    /// Apply a new user's order - update the proper ledger
    /// and return a new transaction if applicable.
    ///
    /// NOTE: We do not want to use anything like "Transaction
    /// Observer" here. The only way the transaction may occure is by
    /// applying some user action, so it's sufficient to
    /// check for any new transaction right after a user's
    /// new order. If there is a new transaction
    /// - handle it right away.
    ///
    /// This approach **reduces the coupling** of the system's components.
    ///
    pub fn handle_user_order(&mut self, order: Order) -> Option<Transaction> {
        match order {
            Order::Buy(_, product) => match product {
                Product::Apple => {
                    self.apples -= 1;
                    if self.apples >= 0 {
                        return Some(Transaction(Product::Apple));
                    }
                }
                Product::Pear => {
                    self.pears -= 1;
                    if self.pears >= 0 {
                        return Some(Transaction(Product::Pear));
                    }
                }
                Product::Tomato => {
                    self.tomatoes -= 1;
                    if self.tomatoes >= 0 {
                        return Some(Transaction(Product::Tomato));
                    }
                }
                Product::Potato => {
                    self.potatoes -= 1;
                    if self.potatoes >= 0 {
                        return Some(Transaction(Product::Potato));
                    }
                }
                Product::Onion => {
                    self.onions -= 1;
                    if self.onions >= 0 {
                        return Some(Transaction(Product::Onion));
                    }
                }
            },
            Order::Sell(_, product) => match product {
                Product::Apple => {
                    self.apples += 1;
                    if self.apples <= 0 {
                        return Some(Transaction(Product::Apple));
                    }
                }
                Product::Pear => {
                    self.pears += 1;
                    if self.pears <= 0 {
                        return Some(Transaction(Product::Pear));
                    }
                }
                Product::Tomato => {
                    self.tomatoes += 1;
                    if self.tomatoes <= 0 {
                        return Some(Transaction(Product::Tomato));
                    }
                }
                Product::Potato => {
                    self.potatoes += 1;
                    if self.potatoes <= 0 {
                        return Some(Transaction(Product::Potato));
                    }
                }
                Product::Onion => {
                    self.onions += 1;
                    if self.onions <= 0 {
                        return Some(Transaction(Product::Onion));
                    }
                }
            },
        };
        None
    }
}
