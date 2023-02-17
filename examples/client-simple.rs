use diagramer::client::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let session = client.new_session("http://localhost:8000").await;
    println!("New session url {}", session.session_url()); 
    session.add_link("a", "b", Some("Request")).await;
    session.add_link("b", "c", Some("Forward")).await;
    session.add_link("c", "a", Some("Response")).await;
}
