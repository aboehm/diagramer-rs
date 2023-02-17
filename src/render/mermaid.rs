use crate::data;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Responder)]
#[response(status = 200)]
pub struct Document(String);

impl From<&data::SessionInner> for Document {
    fn from(session: &data::SessionInner) -> Self {
        let mut definition = String::new();
        definition.push_str("sequenceDiagram\n");

        let mut parties = session.parties.iter().map(|i| &i.0).collect::<Vec<_>>();
        parties.sort();

        for party in parties.iter() {
            let name = party.name.as_str();
            definition.push_str(&format!(
                    "  participant {name} as {}\n",
                    if let Some(label) = party.label.as_deref() { label } else { name }
                    ));
        }

        for link in &session.links {
            let from = link.from.name.as_str().replace("\n", "<br>");
            let to = link.to.name.as_str().replace("\n", "<br>");
            definition.push_str(&format!(
                    "  {from} ->> {to}: {}\n",
                    if let Some(label) = link.label.as_deref() { label } else { "" }
                    ));
        }

        Document(definition)
    }
}

impl Into<String> for Document {
    fn into(self) -> String {
        self.0
    }
}

