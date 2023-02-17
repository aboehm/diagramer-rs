use diagramer::{
    client::Client,
    data::Sessions,
    server::serve,
};
use std::time::Instant;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let _ = serve(Sessions::new()).launch().await.unwrap();
    });

    let client = Client::new();
    let session = client.new_session("http://localhost:8000").await;

    println!("New session url {}", session.session_url());

    let start = Instant::now();
    const COUNT: usize = 10000;
    for i in 0..COUNT {
        let from = format!("node-{}", i % 100);
        let to = format!("node-{}", (i+1) % 100);
        let label = format!("Message #{i}");
        session.add_link(&from, &to, Some(&label)).await;
    }
    let took = start.elapsed();

    println!("Add phase took {} ms, {:.2} requests/ms",
        took.as_millis(),
        COUNT as f32/took.as_millis() as f32
        );

    let request_start = Instant::now();
    let _ = session.mermaid().await;
    println!("Mermaid generation took {} ms", request_start.elapsed().as_millis());
}
