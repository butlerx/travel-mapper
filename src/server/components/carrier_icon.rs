use leptos::prelude::*;

const fn fallback_icon(travel_type: &str) -> &str {
    match travel_type.as_bytes() {
        b"air" => "/static/icons/plane.svg",
        b"rail" => "/static/icons/train.svg",
        b"boat" => "/static/icons/boat.svg",
        _ => "/static/icons/transport.svg",
    }
}

/// Maps a carrier name (lowercased) to its website domain for logo lookup.
///
/// Google's Favicon API (`google.com/s2/favicons?domain={domain}&sz=64`)
/// requires a domain, not a company name.  This table bridges the gap for
/// operators that appear in `TripIt` and manual entry data.  When a carrier
/// isn't listed here the component falls back to the generic travel-type SVG.
#[must_use]
fn carrier_domain(carrier: &str) -> Option<&'static str> {
    let key = carrier.trim();
    // The match is case-insensitive (caller lowercases).
    match key {
        // Airlines
        "aer lingus" => Some("aerlingus.com"),
        "aeroflot" => Some("aeroflot.ru"),
        "air france" => Some("airfrance.com"),
        "air canada" => Some("aircanada.com"),
        "alaska airlines" => Some("alaskaair.com"),
        "american airlines" => Some("aa.com"),
        "asiana airlines" | "asiana" => Some("flyasiana.com"),
        "british airways" => Some("britishairways.com"),
        "cathay pacific" => Some("cathaypacific.com"),
        "delta" | "delta air lines" => Some("delta.com"),
        "easyjet" => Some("easyjet.com"),
        "emirates" => Some("emirates.com"),
        "etihad" | "etihad airways" => Some("etihad.com"),
        "finnair" => Some("finnair.com"),
        "iberia" => Some("iberia.com"),
        "ita airways" | "ita" => Some("ita-airways.com"),
        "japan airlines" | "jal" => Some("jal.com"),
        "jetblue" => Some("jetblue.com"),
        "kenmore air" => Some("kenmoreair.com"),
        "klm" => Some("klm.com"),
        "korean air" => Some("koreanair.com"),
        "lufthansa" => Some("lufthansa.com"),
        "norwegian" | "norwegian air" => Some("norwegian.com"),
        "qatar airways" | "qatar" => Some("qatarairways.com"),
        "ryanair" => Some("ryanair.com"),
        "sas" | "scandinavian airlines" => Some("flysas.com"),
        "singapore airlines" => Some("singaporeair.com"),
        "southwest" | "southwest airlines" => Some("southwest.com"),
        "swiss" | "swiss international" => Some("swiss.com"),
        "tap" | "tap portugal" | "tap air portugal" => Some("flytap.com"),
        "turkish airlines" => Some("turkishairlines.com"),
        "united" | "united airlines" => Some("united.com"),
        "virgin atlantic" => Some("virginatlantic.com"),
        "vueling" => Some("vueling.com"),
        "wizz air" | "wizzair" => Some("wizzair.com"),

        // Rail
        "eurostar" => Some("eurostar.com"),
        "thalys" => Some("thalys.com"),
        "sncf" => Some("sncf.com"),
        "trenitalia" => Some("trenitalia.com"),
        "italo" => Some("italotreno.it"),
        "db" | "deutsche bahn" | "intercity express" => Some("bahn.de"),
        "obb" | "öbb" => Some("oebb.at"),
        "sbb" | "cff" | "ffs" => Some("sbb.ch"),
        "ns" | "nederlandse spoorwegen" => Some("ns.nl"),
        "sj" => Some("sj.se"),
        "renfe" => Some("renfe.com"),
        "cp" | "comboios de portugal" => Some("cp.pt"),
        "irish rail" | "iarnród éireann" | "iarnrod eireann" | "dart" => Some("irishrail.ie"),
        "avanti" | "avanti west coast" => Some("avantiwestcoast.co.uk"),
        "lner" => Some("lner.co.uk"),
        "gwr" | "great western railway" => Some("gwr.com"),
        "scotrail" => Some("scotrail.co.uk"),
        "southeastern" => Some("southeasternrailway.co.uk"),
        "northern" | "northern trains" => Some("northernrailway.co.uk"),
        "crosscountry" => Some("crosscountrytrains.co.uk"),
        "transpennine" | "transpennine express" => Some("tpexpress.co.uk"),
        "east midlands railway" | "emr" => Some("eastmidlandsrailway.co.uk"),
        "amtrak" => Some("amtrak.com"),
        "via rail" | "via" => Some("viarail.ca"),
        "korail" => Some("letskorail.com"),
        "jr" | "japan rail" => Some("jrpass.com"),
        "regiojet" | "regiojet train" => Some("regiojet.com"),
        "glacier express" => Some("glacierexpress.ch"),

        // Ferry / Boat
        "stena line" | "stena" => Some("stenaline.com"),
        "irish ferries" => Some("irishferries.com"),
        "brittany ferries" => Some("brittany-ferries.co.uk"),
        "p&o ferries" | "p&o" => Some("poferries.com"),
        "dfds" => Some("dfds.com"),
        "viking line" => Some("vikingline.com"),
        "tallink" | "tallink silja" => Some("tallink.com"),
        "color line" => Some("colorline.com"),
        "fjord line" => Some("fjordline.com"),
        "corsica ferries" => Some("corsica-ferries.co.uk"),
        "moby" | "moby lines" => Some("moby.it"),
        "tirrenia" => Some("tirrenia.it"),
        "grimaldi lines" | "grimaldi" => Some("grimaldi-lines.com"),
        "condor ferries" | "condor" => Some("condorferries.co.uk"),
        "wightlink" => Some("wightlink.co.uk"),
        "caledonian macbrayne" | "calmac" => Some("calmac.co.uk"),

        // Bus / Transport
        "flixbus" | "flix" => Some("flixbus.com"),
        "greyhound" => Some("greyhound.com"),
        "national express" => Some("nationalexpress.com"),
        "megabus" => Some("megabus.com"),
        "bus eireann" | "bus éireann" => Some("buseireann.ie"),
        "eurolines" => Some("eurolines.eu"),
        "ônibus" => Some("clickbus.com.br"),
        "ouigo" => Some("ouigo.com"),

        _ => None,
    }
}

fn carrier_icon_url(carrier: &str, travel_type: &str) -> String {
    if carrier.is_empty() {
        return fallback_icon(travel_type).to_owned();
    }

    let lower = carrier.to_lowercase();
    if let Some(domain) = carrier_domain(&lower) {
        return format!("https://www.google.com/s2/favicons?domain={domain}&sz=64");
    }

    fallback_icon(travel_type).to_owned()
}

/// Displays a carrier logo for a travel journey.
///
/// Uses the Google Favicon API keyed by operator domain (looked up from a
/// static table), falling back to a generic travel-type SVG when the carrier
/// is unknown or the favicon fails to load.
#[component]
pub fn CarrierIcon(
    #[prop(into)] carrier: String,
    #[prop(into)] travel_type: String,
    #[prop(default = 24)] size: u32,
) -> impl IntoView {
    let fallback = fallback_icon(&travel_type).to_owned();
    let src = carrier_icon_url(&carrier, &travel_type);
    let is_cdn = src != fallback;

    let size_str = size.to_string();
    let onerror = if is_cdn {
        format!("this.onerror=null;this.src='{fallback}'")
    } else {
        String::new()
    };
    let alt = if carrier.is_empty() {
        travel_type.clone()
    } else {
        carrier.clone()
    };

    view! {
        <img
            class="carrier-icon"
            src=src
            alt=alt
            width=size_str.clone()
            height=size_str
            onerror=onerror
            loading="lazy"
        />
    }
}
