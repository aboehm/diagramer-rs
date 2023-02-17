use crate::data::{self, Sessions, SessionInner};

#[cfg(feature = "mermaid")]
use crate::render::mermaid::{Document as MermaidDocument};
#[cfg(feature = "svg")]
use crate::render::svg::{Document as SvgDocument};

use chrono::Utc;
use rocket::{
    State,
    serde::json::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Responder)]
pub enum ErrorKind {
    #[response(status = 404)]
    NotFound(Json<ErrorResponse>),
}

impl ErrorKind {
    pub fn not_found(id: u64, cause: &str) -> Self {
        ErrorKind::NotFound(Json(ErrorResponse {
            id,
            status: "ERROR".to_string(),
            cause: cause.to_string(),
        }))
    }
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub id: u64,
    pub cause: String,
}

#[derive(Deserialize, Serialize)]
pub struct Link {
    pub timestamp: u64,
    pub from: String,
    pub to: String,
    pub label: Arc<Option<String>>,
    pub id: u64,
}

#[derive(Deserialize, Serialize)]
pub struct Session {
    pub id: u64,
    pub links: Vec<Link>,
    pub last_link: u64,
    #[cfg(feature = "mermaid")]
    pub mermaid_url: String,
    #[cfg(feature = "svg")]
    pub svg_url: String,
}

impl From<&data::SessionInner> for Session {
    fn from(session: &data::SessionInner) -> Session {
        Session {
            id: session.id,
            links: session.links.iter()
                .cloned()
                .map(From::<Arc<data::Link>>::from)
                .collect(),
            last_link: session.links.iter()
                .last()
                .map(|i| i.id)
                .unwrap_or(0),
            #[cfg(feature = "mermaid")]
            mermaid_url: uri!(get_mermaid(session.id)).to_string(),
            #[cfg(feature = "svg")]
            svg_url: uri!(get_svg(session.id)).to_string(),
        }
    }
}

impl From<Arc<data::Link>> for Link {
    fn from(link: Arc<data::Link>) -> Link {
        Link {
            timestamp: link.timestamp.timestamp_millis().try_into().unwrap(),
            from: link.from.name.to_string(),
            to: link.to.name.to_string(),
            label: link.label.clone(),
            id: link.id,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct NewSessionResponse {
    pub id: String,
    pub uri: String,
}

#[post("/api/new-session")]
pub(crate) fn new_session(sessions: &State<Sessions>) -> Result<Json<NewSessionResponse>, ErrorKind> {
    let session = sessions.new_session();
    let session = session.read().unwrap();
    Ok(Json(NewSessionResponse {
        id: session.id.to_string(),
        uri: uri!(get_session(session.id)).to_string(),
    }))
}

#[get("/api/session/<id>")]
pub(crate) fn get_session(sessions: &State<Sessions>, id: u64) -> Result<Json<Session>, ErrorKind> {
    let session = sessions.get(id)
        .ok_or(ErrorKind::not_found(id, "Session not exists"))?;
    let session = session.read().unwrap();
    let session: Session = From::<&SessionInner>::from(&*session);
    Ok(Json(session))
}

#[derive(Deserialize, Serialize)]
pub struct AddLinkRequest<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub label: Option<&'a str>,
}

#[derive(Deserialize, Serialize)]
pub struct AddLinkResponse {
    pub link_id: u64,
}

#[post("/api/session/<id>/links", data = "<body>")]
pub(crate) fn add_link<'a>(sessions: &State<Sessions>, id: u64, body: Json<AddLinkRequest<'a>>) -> Result<Json<AddLinkResponse>, ErrorKind> {
    let session = sessions.get(id).ok_or(ErrorKind::not_found(id, "Session doesn't exist"))?;
    let mut session = session.write().unwrap();
    Ok(Json(AddLinkResponse {
        link_id: session.add_link(Utc::now(), body.from, body.to, body.label)
    }))
}

#[get("/api/session/<id>/links")]
pub(crate) fn get_links<'a>(sessions: &State<Sessions>, id: u64) -> Result<Json<Vec<Link>>, ErrorKind> {
    let session = sessions.get(id).ok_or(ErrorKind::not_found(id, "Session doesn't exist"))?;
    let session = session.read().unwrap();
    Ok(Json(session.links.iter().cloned().map(|link| link.into()).collect()))
}

#[derive(Deserialize, Serialize)]
pub struct EventResponse {
    pub highest_link_id: u64,
    pub new_links: Vec<Link>,
    pub events_url: String,
}

#[get("/api/session/<id>/events/<link_id>")]
pub(crate) fn get_events<'a>(sessions: &State<Sessions>, id: u64, link_id: u64) -> Result<Json<EventResponse>, ErrorKind> {
    let session = sessions.get(id).ok_or(ErrorKind::not_found(id, "Session doesn't exist"))?;
    let session = session.read().unwrap();
    let (highest_link_id, new_links) = session.links_above_id(link_id);
    Ok(Json(EventResponse {
        highest_link_id,
        new_links: new_links.iter().cloned().map(|link| link.into()).collect(),
        events_url: uri!(get_events(id, highest_link_id)).to_string(),
    }))
}

#[cfg(feature = "mermaid")]
#[get("/api/session/<id>/mermaid")]
pub(crate) fn get_mermaid(sessions: &State<Sessions>, id: u64) -> Result<MermaidDocument, ErrorKind> {
    let session = sessions.get(id).ok_or(ErrorKind::not_found(id, "Session not exists"))?;
    let session = session.read().unwrap();
    let mermaid: MermaidDocument = Into::<MermaidDocument>::into(&*session);
    Ok(mermaid)
}

#[cfg(feature = "svg")]
#[get("/api/session/<id>/svg")]
pub(crate) fn get_svg(sessions: &State<Sessions>, id: u64) -> Result<SvgDocument, ErrorKind> {
    let session = sessions.get(id).ok_or(ErrorKind::not_found(id, "Session not exists"))?;
    let session = session.read().unwrap();
    let doc: SvgDocument = Into::<SvgDocument>::into(&*session);
    Ok(doc)
}

