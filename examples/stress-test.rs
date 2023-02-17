use chrono::Utc;
use diagramer::data::Sessions;
use std::time::Instant;

fn main() {
    let sessions = Sessions::new();
    let session = sessions.new_session();
    let id = {
        let session = session.read().unwrap();
        session.id
    };
    println!("New session {}", id);

    let start = Instant::now();
    const COUNT: usize = 1000000;
    for i in 0..COUNT {
        let session = sessions.get(id).unwrap();
        let mut session = session.write().unwrap();

        let from = format!("node-{}", i % 100);
        let to = format!("node-{}", (i+1) % 100);
        let label = format!("Message #{i}");
        session.add_link(Utc::now(), &from, &to, Some(&label));
    }
    let took = start.elapsed();
    println!("Add phase took {} ms, {:.2} requests/ms",
        took.as_millis(),
        COUNT as f32/took.as_millis() as f32);
}
