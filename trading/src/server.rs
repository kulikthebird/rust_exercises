/// Author: Tomasz Kulik
/// 
/// 

use crate::ledger::Ledger;
use crate::order::{Order, UserId};
use crate::transaction::{Product, Transaction};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// This structure represents a single event that may occure
/// in the system. There are two possible event types:
/// - Order - created by the user
/// - UserLogin - created once a new user have logged in
#[derive(Debug)]
enum Event {
    Order(Order),
    UserLogin(UserId, WriteHalf<TcpStream>),
}

/// The server handles the incoming connections and notifies
/// every user about the transaction that took place in
/// the market.
pub struct Server {
    users: HashMap<UserId, WriteHalf<TcpStream>>,
}

impl Server {
    /// Create an async server.
    ///
    pub fn new() -> Server {
        Server {
            users: HashMap::new(),
        }
    }

    /// Generates the server's future.
    ///
    pub async fn start(&mut self, ledger: &mut Ledger, interface: String) -> anyhow::Result<()> {
        // Bind to the given interface
        let listener = TcpListener::bind(interface.clone()).await?;
        println!("Listening on: {}", interface);

        // Create a channel to establish a communication between the user handler
        // and the event handler.
        let (event_notification_sender, event_notification_receiver) = channel::<Event>(10000); // Maybe channel buffer size 1 is sufficient?

        futures::try_join!(
            Server::user_handler(listener, event_notification_sender),
            self.event_handler(event_notification_receiver, ledger)
        )?;
        Ok(())
    }

    /// Handles the user's input
    ///
    /// The goals of this method are:
    /// - accepting every incoming connections
    /// - parsing the user's raw input into the sequence
    ///   of orders and sending them to the event handler.
    async fn user_handler(
        listener: TcpListener,
        event_notification_sender: Sender<Event>,
    ) -> anyhow::Result<()> {
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("User connected ({})", addr);

            // This should be a unique for every user.
            // It's ok to use the port number for this purpose.
            let user_id = addr.port();

            // Split the stream into the reader and the writer.
            // Reader is sent to the new async task and will be
            // used to receive the orders from the user.
            // Writer is used by the event handler to inform
            // the user about the transactions and sending ACK msgs.
            let (mut reader, writer) = tokio::io::split(stream);

            // Store the writer using notification handler.
            event_notification_sender
                .send(Event::UserLogin(user_id, writer))
                .await?;

            // Handle users' input within the async loop
            let event_notification_sender = event_notification_sender.clone();
            tokio::spawn(async move {
                loop {
                    let mut buf = [0; 1024];
                    let n = match reader.read(&mut buf).await {
                        // socket closed
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            println!("Error occured: {}", e);
                            return;
                        }
                    };

                    // Parse the entire user input
                    // Iterate over all the lines and parse the orders.
                    let parsed_orders = std::str::from_utf8(&buf[0..n])
                        .map_err(|e| println!("The user input format is not a valid UTF-8: {}", e))
                        .map(|input| {
                            input.lines().filter_map(|input| {
                                Order::new_order_form_str(user_id, input)
                                    .map_err(|e| println!("Unknown user command: {}", e))
                                    .ok()
                            })
                        })
                        .ok();

                    // If there are new orders, send them to the event handler.
                    if let Some(orders) = parsed_orders {
                        for order in orders {
                            event_notification_sender
                                .send(Event::Order(order))
                                .await
                                .expect("There is a problem with the event notification channel");
                        }
                    }
                }
            });
        }
    }

    /// Receives all events sent by the user handler
    ///
    /// This method handles the system events i.e.:
    /// - updates the ledger accordingly to the new orders
    /// - notifies users about new transactions
    /// - sends ACK messages in response to the orders
    async fn event_handler(
        &mut self,
        mut event_notification_receiver: Receiver<Event>,
        ledger: &mut Ledger,
    ) -> anyhow::Result<()> {
        loop {
            let event = event_notification_receiver.recv().await;
            match event {
                Some(Event::Order(order)) => {
                    println!("{}", order);
                    let (user_id, product) = match &order {
                        Order::Buy(user_id, product) => (user_id, product),
                        Order::Sell(user_id, product) => (user_id, product),
                    };
                    if let Some(writer) = self.users.get_mut(&user_id) {
                        if let Err(_) = writer
                            .write_all(format!("ACK:{}\n", product).as_bytes())
                            .await
                        {
                            println!("Removing user with ID: {}", user_id);
                            self.users.remove(&user_id);
                        }
                    }
                    let maybe_transaction = ledger.handle_user_order(order);
                    if let Some(transaction) = maybe_transaction {
                        let Transaction(product) = &transaction;
                        println!("trade ({})", product);
                        self.notify_all_users_about_transaction(transaction).await;
                    }
                }
                Some(Event::UserLogin(user_id, writer)) => {
                    self.users.insert(user_id, writer);
                }
                _ => {
                    println!("There is a problem with the event notification channel!");
                }
            };
        }
    }

    /// Sends a transaction notification to each stored users
    ///
    /// If the user is not reachable anymore - the method
    /// removes it from the set.
    async fn notify_all_users_about_transaction(&mut self, transaction: Transaction) {
        let mut users_to_remove: Vec<UserId> = vec![];
        for (user_id, writer) in &mut self.users {
            if let Err(_) = match transaction {
                Transaction(Product::Apple) => writer.write_all(b"TRADE:APPLE\n"),
                Transaction(Product::Pear) => writer.write_all(b"TRADE:PEAR\n"),
                Transaction(Product::Tomato) => writer.write_all(b"TRADE:TOMATO\n"),
                Transaction(Product::Potato) => writer.write_all(b"TRADE:POTATO\n"),
                Transaction(Product::Onion) => writer.write_all(b"TRADE:ONION\n"),
            }
            .await
            {
                users_to_remove.push(*user_id);
            }
        }
        for user_id in users_to_remove {
            println!("Removing user with ID: {}", user_id);
            self.users.remove(&user_id);
        }
    }
}
