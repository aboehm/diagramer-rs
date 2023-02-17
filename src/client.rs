use crate::api;

use reqwest::{Client as ReqwestClient};

#[derive(Clone)]
pub struct Client(ReqwestClient);

pub const USER_AGENT: &str = "seq-diag-svc";

impl Client {
    pub fn new() -> Self {
        let client = ReqwestClient::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap()
            ;
        Client(client)        
    }

    pub async fn new_session(&self, url: &str) -> Session {
        let resp = self.0.post(format!("{url}{}", uri!(api::new_session())))
            .send()
            .await
            .unwrap()
            .json::<api::NewSessionResponse>()
            .await
            .unwrap();

        let id = resp.id.parse().unwrap();

        Session {
            client: self.0.clone(),
            service_url: url.to_string(),
            session_url: format!("{url}{}", uri!(api::get_session(id))),
            add_link_url: format!("{url}{}", uri!(api::add_link(id))),
            #[cfg(feature = "mermaid")]
            get_mermaid_url: format!("{url}{}", uri!(api::get_mermaid(id))),
            #[cfg(feature = "svg")]
            get_svg_url: format!("{url}{}", uri!(api::get_svg(id))),
            id,
        }
    }
}


pub struct Session {
    client: ReqwestClient,
    id: u64,
    service_url: String,
    session_url: String,
    add_link_url: String,
    #[cfg(feature = "mermaid")]
    get_mermaid_url: String,
    #[cfg(feature = "svg")]
    get_svg_url: String,
}

impl Session {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn service_url(&self) -> &str {
        self.service_url.as_str()
    }

    pub fn session_url(&self) -> &str {
        self.session_url.as_str()
    }

    pub async fn add_link(&self, from: &str, to: &str, label: Option<&str>) {
        self.client.post(&self.add_link_url)
            .json(&api::AddLinkRequest { from, to, label })
            .send()
            .await
            .unwrap();
    }

    #[cfg(feature = "mermaid")]
    pub async fn mermaid(&self) -> String {
        self.client.get(&self.get_mermaid_url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    #[cfg(feature = "svg")]
    pub async fn svg(&self) -> String {
        self.client.get(&self.get_svg_url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }
}
