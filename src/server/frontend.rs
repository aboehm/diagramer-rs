#[allow(unused_imports)]
use crate::{api, data::Sessions};

use rocket_include_tera::{EtagIfNoneMatch, TeraContextManager, TeraResponse};

use chrono::Utc;
use rocket::{
    State,
    http::ContentType,
    response::Redirect,
};
use std::{
    collections::HashMap,
    ops::Deref
};

#[get("/")]
pub(crate) fn index(tera_cm: &State<TeraContextManager>, etag_if_none_match: EtagIfNoneMatch) -> TeraResponse {
    let mut context = HashMap::new();
    context.insert("new_session_url", uri!(new_session()).to_string());
    context.insert("example_url", uri!(new_example()).to_string());
    tera_response!(tera_cm, etag_if_none_match, "index", context)
}

#[get("/styles.css")]
pub(crate) fn styles_css() -> (ContentType, &'static str) {
    (ContentType::CSS, include_str!(concat!(std::env!("CARGO_MANIFEST_DIR"), "/static/styles.css")))
}

#[get("/script.js")]
pub(crate) fn script_js() -> rocket::response::content::RawJavaScript<&'static str> {
    const JS: &'static str = include_str!(concat!(std::env!("CARGO_MANIFEST_DIR"), "/static/script.js"));
    rocket::response::content::RawJavaScript(JS)
}

#[post("/frontend/new-session")]
pub(crate) fn new_session(sessions: &State<Sessions>) -> Redirect {
    let session = sessions.new_session();
    let session = session.read().unwrap();
    Redirect::to(uri!(view(session.id)))
}

#[get("/frontend/<id>")]
pub(crate) fn view(
    tera_cm: &State<TeraContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
    sessions: &State<Sessions>,
    id: u64) -> Result<TeraResponse, &'static str> {
    let session = sessions.get(id).ok_or("Session not exists")?;
    let session = session.read().unwrap();

    #[allow(unused_variables)]
    let session = session.deref();

    let mut context = HashMap::new();
    context.insert("events_url", uri!(api::get_events(id, 0)).to_string());
    context.insert("session_id", id.to_string());
    context.insert("view_url", uri!(view(id)).to_string());

    context.insert("add_link_api_url", uri!(api::add_link(id)).to_string());
    context.insert("session_api_url", uri!(api::get_session(id)).to_string());

    #[cfg(feature = "mermaid")]
    {
        use crate::render::mermaid::Document;
        context.insert("mermaid_doc", Document::from(session).into());
        context.insert("mermaid_url", uri!(api::get_mermaid(id)).to_string());
    }

    #[cfg(feature = "svg")]
    context.insert("svg_url", uri!(api::get_svg(id)).to_string());

    Ok(tera_response!(tera_cm, etag_if_none_match, "session-live-view", context))
}

#[get("/frontend/example")]
pub(crate) fn new_example(sessions: &State<Sessions>) -> Redirect {
    let session = sessions.new_session();
    let mut session = session.write().unwrap();
    let now = Utc::now();
    session.add_link(now, "a", "b", Some("Request"));
    session.add_link(now, "b", "c", Some("Forward"));
    session.add_link(now, "c", "a", Some("Response"));
    Redirect::to(uri!(view(session.id)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::server::test::tester;
    use rocket::http::{ContentType, Status};
    use std::ops::Deref;

    #[tokio::test]
    async fn redirect_for_example() {
        let (sessions, client) = tester().await;
        let response = client.get(uri!(new_example())).dispatch().await;
        assert_eq!(response.status(), Status::SeeOther);
        let sessions = sessions.sessions.read().unwrap();
        let session = sessions.deref().values().next().unwrap();
        let session = session.read().unwrap();
        assert_eq!(3, session.parties.len());
        assert_eq!(3, session.links.len());
    }

    #[tokio::test]
    async fn redirect_for_new_session() {
        let (sessions, client) = tester().await;
        let response = client.post(uri!(new_session())).dispatch().await;
        assert_eq!(response.status(), Status::SeeOther);
        let sessions = sessions.sessions.read().unwrap();
        let session = sessions.deref().values().next().unwrap();
        let session = session.read().unwrap();
        assert_eq!(0, session.parties.len());
        assert_eq!(0, session.links.len());
    }

    #[tokio::test]
    async fn get_view() {
        let (sessions, client) = tester().await;
        let id = {
            let session = sessions.new_session();
            let session = session.read().unwrap();
            session.id
        };
        let response = client.get(uri!(view(id))).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[tokio::test]
    async fn add_link_for_existing_session() {
        let (sessions, client) = tester().await;
        let session = sessions.new_session();
        let id = { session.read().unwrap().id };
        let response = client.post(uri!(add_link(id)))
            .body("from=from&to=to&label=label")
            .header(ContentType::Form)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::SeeOther);
        {
            let session_read = session.read().unwrap();
            assert_eq!(2, session_read.deref().parties.len());
            assert_eq!(1, session_read.deref().links.len());
        }
    }
}
