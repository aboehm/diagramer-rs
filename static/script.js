const LANE_WIDTH = 200;
const INTER_LANE_SPACE = LANE_WIDTH / 10;
const STEP_HEIGHT = 100;
const SPACER_SIZE = LANE_WIDTH/10;
const PARTY_TEXT_OFFSET = 50;
const LINKS_START_HEIGHT = 100;
const LINK_TEXT_OFFSET = 20;
const LINK_LINE_OFFSET = 70;
const ARROW_SIZE = SPACER_SIZE / 2;

var x = SPACER_SIZE;
var y = SPACER_SIZE;
var party_pos_map = {};
var party_end_x = SPACER_SIZE;
var party_end_y = LINKS_START_HEIGHT;

function update_view_box() {
    var svg_doc = document.getElementById("svg-doc");
    svg_doc.setAttribute("viewBox", "0 0 " + (party_end_x) + " " + (party_end_y + 2*SPACER_SIZE));
    Object.keys(party_pos_map).forEach((party) => {
        var rect = document.getElementById("svg_party_rect:" + party);
        rect.setAttribute("height", party_end_y + SPACER_SIZE);

        var line = document.getElementById("svg_party_line:" + party);
        line.setAttribute("y2", party_end_y);

        let x = party_pos_map[party];
        var polygon = document.getElementById("svg_party_arrow:" + party);
        polygon.setAttribute("points", "" + 
            (x) + "," + (party_end_y + ARROW_SIZE) + " " +
            (x - ARROW_SIZE/2) + "," + (party_end_y) + " " +
            (x + ARROW_SIZE/2) + "," + (party_end_y));
    });
}

function add_party(party) {
    var svg_doc = document.getElementById("svg-doc");
    if (party_pos_map[party] != undefined) {
        return; 
    }

    var x = party_end_x;
    console.log("Adding party ", party);

    var rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    rect.setAttribute("id", "svg_party_rect:" + party);
    rect.setAttribute("width", LANE_WIDTH);
    rect.setAttribute("height", party_end_y + SPACER_SIZE);
    rect.setAttribute("x", x);
    rect.setAttribute("y", SPACER_SIZE);
    rect.setAttribute("style", "fill:white;stroke:lightgray");
    svg_doc.appendChild(rect);

    var text = document.createElementNS("http://www.w3.org/2000/svg", "text");
    text.setAttribute("x", x + LANE_WIDTH/2);
    text.setAttribute("y", PARTY_TEXT_OFFSET);
    text.setAttribute("fill", "black");
    text.setAttribute("text-anchor", "middle");
    text.appendChild(document.createTextNode(party));
    svg_doc.appendChild(text);

    var line = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line.setAttribute("id", "svg_party_line:" + party);
    line.setAttribute("x1", x + LANE_WIDTH/2)
    line.setAttribute("y1", LINKS_START_HEIGHT)
    line.setAttribute("x2", x + LANE_WIDTH/2)
    line.setAttribute("y2", party_end_y)
    line.setAttribute("style", "stroke:rgb(0,0,0);stroke-width:2");
    svg_doc.appendChild(line);

    var polygon = document.createElementNS("http://www.w3.org/2000/svg", "polygon");
    polygon.setAttribute("id", "svg_party_arrow:" + party);
    polygon.setAttribute("points", "" + 
        (x + LANE_WIDTH/2) + "," + (party_end_y + ARROW_SIZE) + " " +
        (x + LANE_WIDTH/2 - ARROW_SIZE/2) + "," + (party_end_y) + " " +
        (x + LANE_WIDTH/2 + ARROW_SIZE/2) + "," + (party_end_y));
    polygon.setAttribute("style", "fill:black;stroke-width:0");
    svg_doc.appendChild(polygon);

    party_pos_map[party] = x + LANE_WIDTH/2;
    party_end_x += LANE_WIDTH + INTER_LANE_SPACE;
}

function add_link(from, to, label) {
    var svg_doc = document.getElementById("svg-doc");
    add_party(from);
    add_party(to);

    let from_x = party_pos_map[from];
    let to_x = party_pos_map[to];
    let direction = to_x > from_x ? 1 : -1;
    var y = party_end_y;

    var text = document.createElementNS("http://www.w3.org/2000/svg", "text");
    text.setAttribute("x", to_x - direction * LANE_WIDTH/2);
    text.setAttribute("y", y + LINK_TEXT_OFFSET);
    text.setAttribute("fill", "black");
    text.setAttribute("text-anchor", "middle");
    text.setAttribute("stroke", "white");
    text.setAttribute("stroke-width", "0.5em");
    text.setAttribute("paint-order", "stroke");
    text.setAttribute("stroke-linejoin", "round");
    text.appendChild(document.createTextNode(label));
    svg_doc.appendChild(text);

    var line = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line.setAttribute("x1", from_x + direction * LANE_WIDTH/2)
    line.setAttribute("y1", y + LINK_LINE_OFFSET)
    line.setAttribute("x2", to_x - direction * LANE_WIDTH/2)
    line.setAttribute("y2", y + LINK_LINE_OFFSET)
    line.setAttribute("style", "stroke:rgb(255,255,255);stroke-width:8");
    svg_doc.appendChild(line);

    line = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line.setAttribute("x1", from_x)
    line.setAttribute("y1", y + LINK_LINE_OFFSET)
    line.setAttribute("x2", to_x)
    line.setAttribute("y2", y + LINK_LINE_OFFSET)
    line.setAttribute("style", "stroke:rgb(0,0,0);stroke-width:2");
    svg_doc.appendChild(line);

    var polygon = document.createElementNS("http://www.w3.org/2000/svg", "polygon");
    polygon.setAttribute("points", "" + 
        (to_x) + ","  + (y + LINK_LINE_OFFSET) + " " +
        (to_x - direction * ARROW_SIZE) + "," + (y - ARROW_SIZE/2 + LINK_LINE_OFFSET) + " " +
        (to_x - direction * ARROW_SIZE) + "," + (y + ARROW_SIZE/2 + LINK_LINE_OFFSET));
    polygon.setAttribute("style", "fill:black;stroke-width:0");
    svg_doc.appendChild(polygon);

    party_end_y += STEP_HEIGHT;
    update_view_box();

    return text;
}

function check_events(events_url, event_view_elm) {
    if (document.getElementById("auto_update").checked) {
        fetch(events_url)
            .then((resp) => resp.json())
            .then((data) => {
                setTimeout(() => check_events(data.events_url, event_view_elm), 1000);
                event_handler(data, event_view_elm);
            });
    } else {
        setTimeout(() => check_events(data.events_url, event_view_elm), 1000);
    }
}

function event_handler(data, event_view_elm) {
    data.new_links.forEach(link => {
        console.log("Adding link", link);
        let link_anchor = add_link(link.from, link.to, link.label);
        link_anchor.scrollIntoView({ behavior: "smooth", inline: "center", block: "center" });
    });
}
