use chrono::{
    DateTime,
    offset::Utc,
};
use ring::rand::{SystemRandom, SecureRandom};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

pub struct Party {
    pub id: u64,
    pub name: String,
    pub label: Option<String>,
}

impl Party {
    pub fn new(name: &str) -> Self {
        Self {
            id: 0,
            name: name.to_string(),
            label: None,
        }
    }
}

impl std::cmp::PartialEq for Party {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id

    }
}

impl std::cmp::Eq for Party {}

impl std::cmp::PartialOrd for Party {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl std::cmp::Ord for Party {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Clone)]
pub struct PartyMappedByName(pub Arc<Party>);

impl std::cmp::PartialEq for PartyMappedByName {
    fn eq(&self, other: &PartyMappedByName) -> bool {
        self.0.name == other.0.name
    }
}

impl std::hash::Hash for PartyMappedByName {
    fn hash<H: std::hash::Hasher>(&self, hash: &mut H) {
        hash.write(self.0.name.as_bytes())
    }
}

impl std::cmp::Eq for PartyMappedByName {}

pub struct Link {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub from: Arc<Party>,
    pub to: Arc<Party>,
    pub label: Arc<Option<String>>,
}

impl Link {
    pub fn new(id: u64, timestamp: DateTime<Utc>, from: Arc<Party>, to: Arc<Party>, label: Option<String>) -> Self {
        Self {
            id, timestamp, from, to,
            label: Arc::new(label),
        }
    }
}

pub struct SessionInner {
    pub id: u64,
    pub links: Vec<Arc<Link>>,

    pub parties: HashSet<PartyMappedByName>,

    pub parties_highest_id: u64,
    pub links_highest_id: u64,
}

impl SessionInner {
    pub fn new(id: u64) -> Self {
        SessionInner {
            id,
            links: vec![],
            parties: HashSet::new(),
            parties_highest_id: 1,
            links_highest_id: 1,
        }
    }

    pub fn links_above_id(&self, id: u64) -> (u64, Vec<Arc<Link>>) {
        let mut links = vec![];
        
        for link in self.links.iter().rev() {
            if link.id <= id {
                break;
            }
            links.insert(0, link.clone());
        }

        (self.links_highest_id, links)
    }

    pub fn add_link(&mut self, timestamp: DateTime<Utc>, from: &str, to: &str, label: Option<&str>) -> u64 {
        let from = self.add_party(from);
        self.parties.insert(PartyMappedByName(from.clone()));

        let to = self.add_party(to);
        self.parties.insert(PartyMappedByName(to.clone()));

        self.links_highest_id += 1;
        self.links.push(Arc::new(Link::new(self.links_highest_id, timestamp, from, to, label.map(ToString::to_string))));
        self.links_highest_id
    }

    pub fn add_party(&mut self, name: &str) -> Arc<Party> {
        let mut party = PartyMappedByName(Arc::new(Party::new(name)));

        if let Some(existing_party) = self.parties.get(&party) {
            return existing_party.0.clone();
        }

        self.parties_highest_id += 1;
        Arc::<Party>::get_mut(&mut party.0).unwrap().id = self.parties_highest_id;
        self.parties.insert(party.clone());
        party.0
    }
}

#[derive(Clone)]
pub struct Sessions {
    pub(crate) sessions: Arc<RwLock<HashMap<u64, Arc<RwLock<SessionInner>>>>>,
}

fn get_random() -> u64 {
    let rng = SystemRandom::new();
    let mut buf = [0; 8];
    rng.fill(&mut buf).unwrap();
    u64::from_be_bytes(buf)
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new_session(&self) -> Arc<RwLock<SessionInner>> {
        let mut sessions = self.sessions.write().unwrap();

        let mut id = get_random();
        while sessions.get(&id).is_some() {
            id = get_random();
        }

        let session = Arc::new(RwLock::new(SessionInner::new(id)));
        sessions.insert(id, session.clone());
        session
    }

    pub fn get(&self, id: u64) -> Option<Arc<RwLock<SessionInner>>> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(&id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_graph() -> Arc<RwLock<SessionInner>> {
        let sessions = Sessions::new();
        let session = sessions.new_session();
        {
            let mut session = session.write().unwrap();
            let now = Utc::now();

            session.add_link(now + chrono::Duration::seconds(1), "a", "b", Some("a->b"));
            session.add_link(now + chrono::Duration::seconds(2), "b", "c", Some("b->c"));
            session.add_link(now + chrono::Duration::seconds(3), "c", "b", Some("c->b"));
            session.add_link(now + chrono::Duration::seconds(4), "b", "a", None);
        }
        session
    }

    #[test]
    fn build_simple_graph() {
        let session = simple_graph();
        let session = session.read().unwrap();
        assert_eq!(3, session.parties.len());
        assert_eq!(4, session.links.len());
        assert_eq!(1, session.links[0].id);
        assert_eq!("a", session.links[0].from.name);
        assert_eq!("b", session.links[0].to.name);
        assert_eq!("a->b", Option::as_ref(&session.links[0].label).unwrap());
        assert_eq!(2, session.links[1].id);
        assert_eq!("b", session.links[1].from.name);
        assert_eq!("c", session.links[1].to.name);
        assert_eq!("b->c", Option::as_ref(&session.links[1].label).unwrap());
        assert_eq!(3, session.links[2].id);
        assert_eq!("c", session.links[2].from.name);
        assert_eq!("b", session.links[2].to.name);
        assert_eq!("c->b", Option::as_ref(&session.links[2].label).unwrap());
        assert_eq!(4, session.links[3].id);
        assert_eq!("b", session.links[3].from.name);
        assert_eq!("a", session.links[3].to.name);
        assert_eq!(None, Option::as_ref(&session.links[3].label));
    }

    #[test]
    fn get_links_above_2_after_two_links() {
        let session = simple_graph();
        let session = session.read().unwrap();
        let id = session.links[1].id;
        assert_eq!(2, id);
        assert_eq!(2, session.links_above_id(2).len());
        assert_eq!(3, session.links_above_id(2)[0].id);
        assert_eq!(4, session.links_above_id(2)[1].id);
        assert!(session.links_above_id(4).is_empty());
    }
}
