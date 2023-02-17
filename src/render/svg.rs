use crate::data;
use serde::{Deserialize, Serialize};
use svg::{
    self,
    Document as SvgDocument,
    node::element::Rectangle,
};

use std::{
    collections::HashMap,
    ops::Deref,
};

#[derive(Deserialize, Serialize, Responder)]
#[response(status = 200, content_type = "image/svg+xml")]
pub struct Document(String);

impl From<&data::SessionInner> for Document {
    fn from(session: &data::SessionInner) -> Self {
        let mut doc = SvgDocument::new();
        let mut party_pos_map = HashMap::new();

        const LANE_WIDTH: i32 = 200;
        const INTER_LANE_SPACE: i32 = LANE_WIDTH / 10;
        const STEP_HEIGHT: i32 = 100;
        const SPACER_SIZE: i32 = LANE_WIDTH/10;
        const PARTY_TEXT_OFFSET: i32 = 50;
        const LINKS_START_HEIGHT: i32 = 100;
        const LINK_TEXT_OFFSET: i32 = 20;
        const LINK_LINE_OFFSET: i32 = 70;
        const ARROW_SIZE: i32 = SPACER_SIZE / 2;

        let mut x: i32 = SPACER_SIZE;
        let mut y: i32 = SPACER_SIZE;
        let max_lane_y: i32 = LINKS_START_HEIGHT + STEP_HEIGHT * session.links.len() as i32;

        let mut parties = session.parties.iter().map(|i| &i.0).collect::<Vec<_>>();
        parties.sort();

        for party in parties.iter() {
            doc = doc.add(Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", LANE_WIDTH)
                .set("height", max_lane_y + SPACER_SIZE)
                .set("style", "fill:white;stroke:lightgray")
                );
            doc = doc.add(svg::node::element::Text::new()
                .set("x", x + LANE_WIDTH/2)
                .set("y", PARTY_TEXT_OFFSET)
                .set("fill", "black")
                .set("text-anchor", "middle")
                .add(svg::node::Text::new(party.name.deref()))
                );
            doc = doc.add(
                svg::node::element::Line::new()
                    .set("x1", x + LANE_WIDTH/2)
                    .set("y1", y + PARTY_TEXT_OFFSET)
                    .set("x2", x + LANE_WIDTH/2)
                    .set("y2", y + max_lane_y - ARROW_SIZE)
                    .set("style", "stroke:rgb(0,0,0);stroke-width:2")
                );
            doc = doc.add(
                svg::node::element::Polygon::new()
                    .set("points", format!(
                            "{},{} {},{} {},{}",
                            x + LANE_WIDTH/2, y + max_lane_y,
                            x + LANE_WIDTH/2 - ARROW_SIZE/2, y + max_lane_y - ARROW_SIZE,
                            x + LANE_WIDTH/2 + ARROW_SIZE/2, y + max_lane_y - ARROW_SIZE,
                            ))
                    .set("style", "fill:black;stroke-width:0")
                );

            party_pos_map.insert(party.name.deref(), x + LANE_WIDTH/2).unwrap_or(x);
            x += LANE_WIDTH + INTER_LANE_SPACE;
        }

        y = LINKS_START_HEIGHT;

        for link in &session.links {
            let from: i32 = *party_pos_map.get(link.from.name.deref()).unwrap();
            let to: i32 = *party_pos_map.get(link.to.name.deref()).unwrap();
            let direction = if to > from { 1 } else { -1 };

            if let Some(label) = link.label.deref() {
                doc = doc.add(
                    svg::node::element::Text::new()
                    .set("x", to - direction * LANE_WIDTH/2)
                    .set("y", y + LINK_TEXT_OFFSET)
                    .set("fill", "black")
                    .set("text-anchor", "middle")
                    .set("stroke", "white")
                    .set("stroke-width", "0.5em")
                    .set("paint-order", "stroke")
                    .set("stroke-linejoin", "round")
                    .add(svg::node::Text::new(label.deref()))
                    );
            }

            doc = doc.add(
                svg::node::element::Line::new()
                    .set("x1", from + direction * LANE_WIDTH/2)
                    .set("y1", y + LINK_LINE_OFFSET)
                    .set("x2", to - direction * (ARROW_SIZE + LANE_WIDTH/2))
                    .set("y2", y + LINK_LINE_OFFSET)
                    .set("style", "stroke:rgb(255,255,255);stroke-width:8")
                );

            doc = doc.add(
                svg::node::element::Line::new()
                    .set("x1", from)
                    .set("y1", y + LINK_LINE_OFFSET)
                    .set("x2", to - direction * ARROW_SIZE)
                    .set("y2", y + LINK_LINE_OFFSET)
                    .set("style", "stroke:rgb(0,0,0);stroke-width:2")
                );

            doc = doc.add(
                svg::node::element::Polygon::new()
                    .set("points", format!(
                            "{},{} {},{} {},{}",
                            to, y + LINK_LINE_OFFSET,
                            to - direction * ARROW_SIZE, y - ARROW_SIZE/2 + LINK_LINE_OFFSET,
                            to - direction * ARROW_SIZE, y + ARROW_SIZE/2 + LINK_LINE_OFFSET,
                            ))
                    .set("style", "fill:black;stroke-width:0")
                );

            y += STEP_HEIGHT;
        }

        doc = doc.set("viewBox", (0, 0, x, max_lane_y + 2*SPACER_SIZE));
        let mut buf = Vec::<u8>::new();
        svg::write(&mut buf, &doc).unwrap();
        Document(String::from_utf8(buf).unwrap())
    }
}

impl Into<String> for Document {
    fn into(self) -> String {
        self.0
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::data::{Sessions};
    use chrono::Utc;

    #[test]
    fn generate() {
        let session = Sessions::new().new_session();
        let mut session = session.write().unwrap();
        let now = Utc::now();
        session.add_link(now, "a", "b", Some("Request"));
        session.add_link(now, "b", "c", Some("Forward"));
        session.add_link(now, "c", "a", Some("Response"));
        let svg_text: Document = session.deref().into();
        assert!(svg_text.0.contains("<svg"));
        assert!(svg_text.0.contains("<text "));
        assert!(svg_text.0.contains("<polygon "));
        assert!(svg_text.0.contains("<rect "));
        assert!(svg_text.0.contains("</svg>"));
    }
}
