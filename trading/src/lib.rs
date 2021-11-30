/// Author: Tomasz Kulik
/// 
///

mod ledger;
mod order;
mod server;
mod transaction;

pub async fn start_server(interface: String) -> anyhow::Result<()> {
    let mut ledger = ledger::Ledger::new();
    let mut server = server::Server::new();
    server
        .start(&mut ledger, interface)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};


    macro_rules! check_product {
        ($client:ident, $client_buf:ident, $product:literal) => {
            $client
                .write_all(format!("BUY:{}\n", $product).as_bytes())
                .await
                .expect("Client error");
            $client
                .write_all(format!("BUY:{}\n", $product).as_bytes())
                .await
                .expect("Client error");
            $client
                .write_all(format!("SELL:{}\n", $product).as_bytes())
                .await
                .expect("Client error");
            $client
                .write_all(format!("SELL:{}\n", $product).as_bytes())
                .await
                .expect("Client error");
            
            // This is crucial for the system to fully receive the server's response
            // in the socket
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let n = match $client.read(&mut $client_buf).await {
                Ok(n) => n,
                _ => panic!("Something's wrong with the socket"),
            };
            let actual_response =
                std::str::from_utf8(&$client_buf[0..n]).expect("Unable to parse server's response");
            assert_eq!(
                actual_response
                    .lines()
                    .filter(|line| line == &format!("ACK:{}", $product))
                    .count(),
                4,
                "{}",
                actual_response
            );
            assert_eq!(
                actual_response
                    .lines()
                    .filter(|line| line == &format!("TRADE:{}", $product))
                    .count(),
                2,
                "{}",
                actual_response
            );
        };
    }

    #[tokio::test]
    async fn test_one_client_all_products() {
        tokio::spawn(start_server("127.0.0.1:8080".to_string()));
        let mut client1 = tokio::net::TcpStream::connect("localhost:8080")
            .await
            .expect("Problem with client1");
        let mut client1_buf = [0; 2048];

        check_product!(client1, client1_buf, "APPLE");
        check_product!(client1, client1_buf, "PEAR");
        check_product!(client1, client1_buf, "TOMATO");
        check_product!(client1, client1_buf, "POTATO");
        check_product!(client1, client1_buf, "ONION");
    }

    #[tokio::test]
    async fn test_multiple_client_all_products() {
        tokio::spawn(start_server("127.0.0.1:8081".to_string()));
        let mut client_buf = [0; 2048];
        let mut client1 = tokio::net::TcpStream::connect("localhost:8081")
            .await
            .expect("Problem with client1");
        let mut client2 = tokio::net::TcpStream::connect("localhost:8081")
            .await
            .expect("Problem with client2");
        let mut client3 = tokio::net::TcpStream::connect("localhost:8081")
            .await
            .expect("Problem with client3");
        let mut client4 = tokio::net::TcpStream::connect("localhost:8081")
            .await
            .expect("Problem with client4");
        let mut client5 = tokio::net::TcpStream::connect("localhost:8081")
            .await
            .expect("Problem with client5");

        check_product!(client1, client_buf, "APPLE");
        check_product!(client2, client_buf, "PEAR");
        check_product!(client3, client_buf, "TOMATO");
        check_product!(client4, client_buf, "POTATO");
        check_product!(client5, client_buf, "ONION");
    }
}
