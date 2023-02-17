#[macro_use]
extern crate rocket;

use diagramer::{
    data::Sessions,
    server::serve,
};

#[launch]
fn launch() -> _ {
    serve(Sessions::new())
}
