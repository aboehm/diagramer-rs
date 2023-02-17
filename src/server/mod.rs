#[cfg(feature = "frontend")]
pub mod frontend;

use crate::{api, data::Sessions};
use rocket::{
    self,
    Build, Rocket,
};

#[cfg(feature = "frontend")]
use rocket_include_tera::TeraResponse;

#[cfg(not(feature = "frontend"))]
#[get("/")]
fn index() -> &'static str {
    concat!(std::env!("CARGO_PKG_NAME"), " v", std::env!("CARGO_PKG_VERSION"))
}

pub fn serve(sessions: Sessions) -> Rocket<Build> {
    let rocket = rocket::build();

    #[cfg(feature = "frontend")]
    let rocket = rocket.attach(TeraResponse::fairing(|tera| {
            tera_resources_initialize!(
                tera,
                "base" => ("templates", "base.html.tera"),
                "index" => ("templates", "index.html.tera"),
                "session-live-view" => ("templates", "session-live-view.html.tera"),
            )
        }));

    let rocket = rocket.manage(sessions);

    #[allow(unused_mut)]
    let mut routes = vec![];
    
    #[cfg(not(feature = "frontend"))]
    routes.append(&mut routes![index]);
    #[cfg(feature = "frontend")]
    routes.append(&mut routes![frontend::index]);

    #[cfg(feature = "frontend")]
    {
        routes.append(&mut routes![
            frontend::new_session,
            frontend::view,
            frontend::styles_css,
            frontend::script_js,
        ]);
            
        routes.append(&mut routes![frontend::new_example]);
    }

    routes.append(&mut routes![
        api::new_session,
        api::get_session,
        api::add_link,
        api::get_links,
        api::get_events,
    ]);
        
    #[cfg(feature = "mermaid")]
    routes.append(&mut routes![api::get_mermaid]);
    #[cfg(feature = "svg")]
    routes.append(&mut routes![api::get_svg]);

    let rocket = rocket.mount("/", routes);

    rocket
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use chrono::Utc;
    use rocket::{
        http::Status,
        local::asynchronous::Client,
    };

    pub(crate) async fn tester() -> (Sessions, Client) {
        let sessions = Sessions::new();
        let rocket = serve(sessions.clone())
            .ignite()
            .await
            .expect("A server");
        let client = Client::tracked(rocket).await.expect("A client");
        (sessions, client)
    }

    #[tokio::test]
    async fn request_new_session() {
        let (sessions, client) = tester().await;
        let response = client.post(uri!(api::new_session())).dispatch().await;
        assert_eq!(response.status(), Status::Ok);

        let response: api::NewSessionResponse = response.into_json().await.unwrap();

        let sessions = sessions.sessions.read().unwrap();
        let (id, _) = sessions.iter().next().unwrap();

        assert_eq!(*id.to_string(), response.id);
        assert_eq!(uri!(api::get_session(*id)).to_string(), response.uri);
    }

    #[tokio::test]
    async fn request_add_link() {
        let (sessions, client) = tester().await;
        let session = sessions.new_session();
        let id = {
            let session = session.read().unwrap();
            session.id
        };

        let body = api::AddLinkRequest { from: "from", to: "to", label: Some("label"), };
        let response = client.post(uri!(api::add_link(id))).json(&body).dispatch().await;
        assert_eq!(Status::Ok, response.status());
    }

    #[tokio::test]
    async fn request_get_link() {
        let (sessions, client) = tester().await;
        let session = sessions.new_session();
        let (id, now) = {
            let mut session = session.write().unwrap();
            let now = Utc::now();
            session.add_link(Utc::now(), "from", "to", Some("label"));
            (session.id, now)
        };

        let response = client.get(uri!(api::get_links(id))).dispatch().await;
        let body: Vec<api::Link> = response.into_json().await.expect("A valid json");
        assert_eq!(1, body.len());
        assert_eq!(now.timestamp_millis(), TryInto::<i64>::try_into(body[0].timestamp).unwrap());
        assert_eq!("from", body[0].from);
        assert_eq!("to", body[0].to);
        assert_eq!(Some("label"), body[0].label.as_deref());
    }

    #[cfg(feature = "mermaid")]
    #[tokio::test]
    async fn mermaid_output() {
        let (sessions, client) = tester().await;
        let id = {
            let session = sessions.new_session();
            let mut session = session.write().unwrap();
            let now = Utc::now();
            session.add_link(now, "a", "b", Some("Request"));
            session.add_link(now, "b", "c", Some("Forward"));
            session.add_link(now, "c", "a", Some("Response"));
            session.id
        };

        let response = client.get(uri!(api::get_mermaid(id))).dispatch().await;
        let _ = response.into_string().await.unwrap();
    }
}
