//! OpenSky Network API client for route verification using ADS-B flight data.
//!
//! This module provides OAuth2 token management, OpenSky flight lookups, and a
//! route verification helper that matches expected flight numbers and airports
//! against OpenSky observations.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::RwLock;

const OPENSKY_API_BASE: &str = "https://opensky-network.org/api";
const OPENSKY_TOKEN_URL: &str =
    "https://auth.opensky-network.org/auth/realms/opensky-network/protocol/openid-connect/token";
const MAX_RETRIES: u32 = 3;
const REQUEST_TIMEOUT_SECS: u64 = 30;
const TOKEN_REFRESH_BUFFER_SECS: u64 = 60;
const FLIGHTS_ALL_WINDOW_SECS: i64 = 2 * 60 * 60;
const MAX_LIFETIME_REQUESTS: u32 = 3_500;

#[derive(Debug, Clone)]
struct TokenState {
    access_token: String,
    expires_at: std::time::Instant,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum OpenSkyError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("OAuth2 token acquisition failed: {status}")]
    TokenError { status: u16 },

    #[error("API returned error status: {status}")]
    ApiError { status: u16 },

    #[error("invalid flight date: {0}")]
    InvalidDate(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("rate limit exceeded")]
    RateLimited,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightRecord {
    pub icao24: String,
    pub first_seen: i64,
    pub est_departure_airport: Option<String>,
    pub last_seen: i64,
    pub est_arrival_airport: Option<String>,
    pub callsign: Option<String>,
    pub est_departure_airport_horiz_distance: Option<i64>,
    pub est_departure_airport_vert_distance: Option<i64>,
    pub est_arrival_airport_horiz_distance: Option<i64>,
    pub est_arrival_airport_vert_distance: Option<i64>,
    pub departure_airport_candidates_count: Option<i64>,
    pub arrival_airport_candidates_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlightVerification {
    pub operated: bool,
    pub est_departure_airport: Option<String>,
    pub est_arrival_airport: Option<String>,
    pub first_seen: Option<i64>,
    pub last_seen: Option<i64>,
    pub callsign: Option<String>,
    pub raw_json: String,
}

#[derive(Clone)]
pub struct OpenSkyClient {
    client_id: String,
    client_secret: String,
    client: reqwest::Client,
    base_url: String,
    token_url: String,
    token: Arc<RwLock<Option<TokenState>>>,
    daily_requests: Arc<AtomicU32>,
}

impl OpenSkyClient {
    #[must_use]
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self::with_urls(
            client_id,
            client_secret,
            reqwest::Client::new(),
            OPENSKY_API_BASE.to_string(),
            OPENSKY_TOKEN_URL.to_string(),
        )
    }

    #[must_use]
    pub fn with_urls(
        client_id: String,
        client_secret: String,
        client: reqwest::Client,
        base_url: String,
        token_url: String,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            client,
            base_url,
            token_url,
            token: Arc::new(RwLock::new(None)),
            daily_requests: Arc::new(AtomicU32::new(0)),
        }
    }

    #[must_use]
    pub fn requests_made(&self) -> u32 {
        self.daily_requests.load(Ordering::Relaxed)
    }

    /// # Errors
    ///
    /// Returns an error if authentication fails, the request fails, or the
    /// response cannot be parsed.
    pub async fn get_flights_for_aircraft(
        &self,
        icao24: &str,
        begin: i64,
        end: i64,
    ) -> Result<Vec<FlightRecord>, OpenSkyError> {
        let url = format!(
            "{}/flights/aircraft?icao24={icao24}&begin={begin}&end={end}",
            self.base_url,
        );
        self.get_flights(&url).await
    }

    /// # Errors
    ///
    /// Returns an error if the window exceeds `OpenSky` limits, authentication
    /// fails, the request fails, or the response cannot be parsed.
    pub async fn get_flights_all(
        &self,
        begin: i64,
        end: i64,
    ) -> Result<Vec<FlightRecord>, OpenSkyError> {
        if end < begin {
            return Ok(Vec::new());
        }
        if end - begin > FLIGHTS_ALL_WINDOW_SECS {
            return Err(OpenSkyError::ApiError { status: 400 });
        }
        let url = format!("{}/flights/all?begin={begin}&end={end}", self.base_url);
        self.get_flights(&url).await
    }

    /// # Errors
    ///
    /// Returns an error if request limits are exceeded, date parsing fails,
    /// upstream calls fail, or response payloads cannot be serialized.
    pub async fn verify_route(
        &self,
        flight_iata: &str,
        flight_date: &str,
        dep_iata: &str,
        arr_iata: &str,
    ) -> Result<Option<FlightVerification>, OpenSkyError> {
        if self.requests_made() >= MAX_LIFETIME_REQUESTS {
            return Err(OpenSkyError::RateLimited);
        }

        let expected_callsign = iata_to_icao_callsign(flight_iata);
        let expected_dep_icao = iata_to_icao_airport(dep_iata);
        let expected_arr_icao = iata_to_icao_airport(arr_iata);

        let date = NaiveDate::parse_from_str(flight_date, "%Y-%m-%d")
            .map_err(|_| OpenSkyError::InvalidDate(flight_date.to_string()))?;
        let start_of_day = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| OpenSkyError::InvalidDate(flight_date.to_string()))?
            .and_utc()
            .timestamp();
        let end_of_day = start_of_day + 86_399;

        let mut candidate: Option<FlightVerification> = None;
        let mut window_start = start_of_day;

        while window_start <= end_of_day {
            let window_end = (window_start + FLIGHTS_ALL_WINDOW_SECS).min(end_of_day);
            let flights = self.get_flights_all(window_start, window_end).await?;

            for flight in flights {
                let flight_callsign = flight
                    .callsign
                    .as_deref()
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_uppercase();
                if !flight_callsign.starts_with(&expected_callsign) {
                    continue;
                }

                let dep_matches = expected_dep_icao
                    .is_none_or(|icao| flight.est_departure_airport.as_deref() == Some(icao));
                let arr_matches = expected_arr_icao
                    .is_none_or(|icao| flight.est_arrival_airport.as_deref() == Some(icao));
                let verification = FlightVerification {
                    operated: dep_matches && arr_matches,
                    est_departure_airport: flight.est_departure_airport.clone(),
                    est_arrival_airport: flight.est_arrival_airport.clone(),
                    first_seen: Some(flight.first_seen),
                    last_seen: Some(flight.last_seen),
                    callsign: flight.callsign.clone(),
                    raw_json: serde_json::to_string(&flight)?,
                };

                if verification.operated {
                    return Ok(Some(verification));
                }
                if candidate.is_none() {
                    candidate = Some(verification);
                }
            }

            if window_end == end_of_day {
                break;
            }
            window_start = window_end + 1;
        }

        Ok(candidate)
    }

    async fn get_flights(&self, url: &str) -> Result<Vec<FlightRecord>, OpenSkyError> {
        let body = self.get_with_retry(url).await?;
        Ok(serde_json::from_slice::<Vec<FlightRecord>>(&body)?)
    }

    async fn get_with_retry(&self, url: &str) -> Result<Vec<u8>, OpenSkyError> {
        let mut attempt: u32 = 0;
        loop {
            attempt += 1;
            self.daily_requests.fetch_add(1, Ordering::Relaxed);
            let token = self.get_access_token().await?;
            let response = self
                .client
                .get(url)
                .bearer_auth(token)
                .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    if status == reqwest::StatusCode::UNAUTHORIZED && attempt <= MAX_RETRIES {
                        tracing::warn!(
                            url,
                            attempt,
                            "token rejected; refreshing token and retrying"
                        );
                        let mut write_guard = self.token.write().await;
                        *write_guard = None;
                        continue;
                    }

                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        if attempt <= MAX_RETRIES {
                            let delay = retry_after_delay(resp.headers())
                                .unwrap_or_else(|| backoff(attempt));
                            tracing::warn!(
                                url,
                                delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                                attempt,
                                "retrying after rate limit response",
                            );
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(OpenSkyError::RateLimited);
                    }

                    if status.is_server_error() && attempt <= MAX_RETRIES {
                        let delay = backoff(attempt);
                        tracing::warn!(
                            url,
                            status = %status,
                            delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                            attempt,
                            "retrying after server error",
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    if !status.is_success() {
                        return Err(OpenSkyError::ApiError {
                            status: status.as_u16(),
                        });
                    }

                    return resp
                        .bytes()
                        .await
                        .map(|body| body.to_vec())
                        .map_err(OpenSkyError::Http);
                }
                Err(err) => {
                    if (err.is_connect() || err.is_timeout()) && attempt <= MAX_RETRIES {
                        let delay = backoff(attempt);
                        tracing::warn!(
                            url,
                            error = %err,
                            delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                            attempt,
                            "retrying after transport error",
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(OpenSkyError::Http(err));
                }
            }
        }
    }

    async fn get_access_token(&self) -> Result<String, OpenSkyError> {
        {
            let read_guard = self.token.read().await;
            if let Some(state) = read_guard.as_ref()
                && state.expires_at
                    > std::time::Instant::now()
                        + std::time::Duration::from_secs(TOKEN_REFRESH_BUFFER_SECS)
            {
                return Ok(state.access_token.clone());
            }
        }

        let mut write_guard = self.token.write().await;
        if let Some(state) = write_guard.as_ref()
            && state.expires_at
                > std::time::Instant::now()
                    + std::time::Duration::from_secs(TOKEN_REFRESH_BUFFER_SECS)
        {
            return Ok(state.access_token.clone());
        }

        let response = self
            .client
            .post(&self.token_url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
            ])
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OpenSkyError::TokenError {
                status: response.status().as_u16(),
            });
        }

        let token_response = response.json::<TokenResponse>().await?;
        let token = token_response.access_token;
        *write_guard = Some(TokenState {
            access_token: token.clone(),
            expires_at: std::time::Instant::now()
                + std::time::Duration::from_secs(token_response.expires_in),
        });
        Ok(token)
    }
}

fn backoff(attempt: u32) -> std::time::Duration {
    std::time::Duration::from_millis(500 * u64::from(2_u32.pow(attempt.saturating_sub(1))))
}

fn retry_after_delay(headers: &reqwest::header::HeaderMap) -> Option<std::time::Duration> {
    let value = headers.get(reqwest::header::RETRY_AFTER)?;
    let as_str = value.to_str().ok()?;
    let seconds = as_str.parse::<u64>().ok()?;
    Some(std::time::Duration::from_secs(seconds))
}

const AIRLINE_IATA_TO_ICAO: [(&str, &str); 85] = [
    ("AA", "AAL"),
    ("AC", "ACA"),
    ("AF", "AFR"),
    ("AI", "AIC"),
    ("AM", "AMX"),
    ("AR", "ARG"),
    ("AS", "ASA"),
    ("AT", "RAM"),
    ("AV", "AVA"),
    ("AY", "FIN"),
    ("AZ", "ITY"),
    ("BA", "BAW"),
    ("BE", "BEE"),
    ("BG", "BBC"),
    ("BI", "RBA"),
    ("BR", "EVA"),
    ("BT", "BTI"),
    ("B6", "JBU"),
    ("CA", "CCA"),
    ("CI", "CAL"),
    ("CM", "CMP"),
    ("CX", "CPA"),
    ("CZ", "CSN"),
    ("DL", "DAL"),
    ("DT", "DTA"),
    ("EK", "UAE"),
    ("EI", "EIN"),
    ("EN", "DLA"),
    ("ET", "ETH"),
    ("EY", "ETD"),
    ("F9", "FFT"),
    ("FI", "ICE"),
    ("FR", "RYR"),
    ("GA", "GIA"),
    ("GF", "GFA"),
    ("G3", "GLO"),
    ("HA", "HAL"),
    ("HO", "DKH"),
    ("HU", "CHH"),
    ("IB", "IBE"),
    ("JL", "JAL"),
    ("JU", "ASL"),
    ("J2", "AHY"),
    ("KA", "HDA"),
    ("KE", "KAL"),
    ("KL", "KLM"),
    ("KM", "AMC"),
    ("KQ", "KQA"),
    ("LA", "LAN"),
    ("LH", "DLH"),
    ("LO", "LOT"),
    ("LS", "EXS"),
    ("LX", "SWR"),
    ("LY", "ELY"),
    ("MH", "MAS"),
    ("MS", "MSR"),
    ("MU", "CES"),
    ("NH", "ANA"),
    ("NZ", "ANZ"),
    ("OA", "OAL"),
    ("OK", "CSA"),
    ("OS", "AUA"),
    ("OU", "CTN"),
    ("PC", "PGT"),
    ("PR", "PAL"),
    ("QF", "QFA"),
    ("QR", "QTR"),
    ("RO", "ROT"),
    ("RJ", "RJA"),
    ("SK", "SAS"),
    ("SN", "BEL"),
    ("SQ", "SIA"),
    ("SU", "AFL"),
    ("SV", "SVA"),
    ("TK", "THY"),
    ("TP", "TAP"),
    ("UA", "UAL"),
    ("UL", "ALK"),
    ("U2", "EZY"),
    ("VY", "VLG"),
    ("VS", "VIR"),
    ("W6", "WZZ"),
    ("WN", "SWA"),
    ("WS", "WJA"),
    ("WY", "OMA"),
];

const AIRPORT_IATA_TO_ICAO: [(&str, &str); 175] = [
    ("ABQ", "KABQ"),
    ("ACC", "DGAA"),
    ("ADB", "LTBJ"),
    ("ADD", "HAAB"),
    ("ADL", "YPAD"),
    ("AKL", "NZAA"),
    ("ALC", "LEAL"),
    ("ALG", "DAAG"),
    ("AMS", "EHAM"),
    ("ANC", "PANC"),
    ("ARN", "ESSA"),
    ("ATH", "LGAV"),
    ("ATL", "KATL"),
    ("AUS", "KAUS"),
    ("AUH", "OMAA"),
    ("BCN", "LEBL"),
    ("BDL", "KBDL"),
    ("BEG", "LYBE"),
    ("BEL", "SBBE"),
    ("BER", "EDDB"),
    ("BFS", "EGAA"),
    ("BGW", "ORBI"),
    ("BHX", "EGBB"),
    ("BKK", "VTBS"),
    ("BLQ", "LIPE"),
    ("BNA", "KBNA"),
    ("BNE", "YBBN"),
    ("BOD", "LFBD"),
    ("BOM", "VABB"),
    ("BOS", "KBOS"),
    ("BRE", "EDDW"),
    ("BRU", "EBBR"),
    ("BSL", "LFSB"),
    ("BUD", "LHBP"),
    ("BUF", "KBUF"),
    ("BWI", "KBWI"),
    ("CAI", "HECA"),
    ("CAN", "ZGGG"),
    ("CCU", "VECC"),
    ("CDG", "LFPG"),
    ("CEB", "RPVM"),
    ("CGK", "WIII"),
    ("CLT", "KCLT"),
    ("CMN", "GMMN"),
    ("CMB", "VCBI"),
    ("CPH", "EKCH"),
    ("CPT", "FACT"),
    ("CRK", "RPLC"),
    ("CSX", "ZGHA"),
    ("CTS", "RJCC"),
    ("CTU", "ZUUU"),
    ("CUN", "MMUN"),
    ("CVG", "KCVG"),
    ("DCA", "KDCA"),
    ("DEL", "VIDP"),
    ("DEN", "KDEN"),
    ("DFW", "KDFW"),
    ("DME", "UUDD"),
    ("DOH", "OTHH"),
    ("DUB", "EIDW"),
    ("DUS", "EDDL"),
    ("DXB", "OMDB"),
    ("EDI", "EGPH"),
    ("EWR", "KEWR"),
    ("EZE", "SAEZ"),
    ("FCO", "LIRF"),
    ("FLL", "KFLL"),
    ("FRA", "EDDF"),
    ("GDL", "MMGL"),
    ("GIG", "SBGL"),
    ("GLA", "EGPF"),
    ("GMP", "RKSS"),
    ("GRU", "SBGR"),
    ("GVA", "LSGG"),
    ("HAM", "EDDH"),
    ("HAN", "VVNB"),
    ("HEL", "EFHK"),
    ("HGH", "ZSHC"),
    ("HKG", "VHHH"),
    ("HND", "RJTT"),
    ("HNL", "PHNL"),
    ("HOU", "KHOU"),
    ("HYD", "VOHS"),
    ("IAH", "KIAH"),
    ("ICN", "RKSI"),
    ("IND", "KIND"),
    ("IST", "LTFM"),
    ("JED", "OEJN"),
    ("JFK", "KJFK"),
    ("JNB", "FAOR"),
    ("KBP", "UKBB"),
    ("KHI", "OPKC"),
    ("KIX", "RJBB"),
    ("KUL", "WMKK"),
    ("KWI", "OKBK"),
    ("LAS", "KLAS"),
    ("LAX", "KLAX"),
    ("LGA", "KLGA"),
    ("LGW", "EGKK"),
    ("LHR", "EGLL"),
    ("LIM", "SPJC"),
    ("LIN", "LIML"),
    ("LIS", "LPPT"),
    ("LPA", "GCLP"),
    ("LTN", "EGGW"),
    ("LYS", "LFLL"),
    ("MAD", "LEMD"),
    ("MAN", "EGCC"),
    ("MCI", "KMCI"),
    ("MCO", "KMCO"),
    ("MCT", "OOMS"),
    ("MEL", "YMML"),
    ("MEM", "KMEM"),
    ("MEX", "MMMX"),
    ("MIA", "KMIA"),
    ("MIL", "LIMC"),
    ("MNL", "RPLL"),
    ("MRS", "LFML"),
    ("MSP", "KMSP"),
    ("MUC", "EDDM"),
    ("MXP", "LIMC"),
    ("NBO", "HKJK"),
    ("NCE", "LFMN"),
    ("NRT", "RJAA"),
    ("OAK", "KOAK"),
    ("OKA", "ROAH"),
    ("OMA", "KOMA"),
    ("ORD", "KORD"),
    ("OSL", "ENGM"),
    ("OTP", "LROP"),
    ("PHL", "KPHL"),
    ("PHX", "KPHX"),
    ("PKX", "ZBAD"),
    ("PMI", "LEPA"),
    ("PRG", "LKPR"),
    ("PVG", "ZSPD"),
    ("QRO", "MMQT"),
    ("RAK", "GMMX"),
    ("RDU", "KRDU"),
    ("RIX", "EVRA"),
    ("RNO", "KRNO"),
    ("RUH", "OERK"),
    ("SAN", "KSAN"),
    ("SAT", "KSAT"),
    ("SEA", "KSEA"),
    ("SFO", "KSFO"),
    ("SGN", "VVTS"),
    ("SHA", "ZSSS"),
    ("SIN", "WSSS"),
    ("SJC", "KSJC"),
    ("SJU", "TJSJ"),
    ("SLC", "KSLC"),
    ("SNN", "EINN"),
    ("STL", "KSTL"),
    ("STR", "EDDS"),
    ("SYD", "YSSY"),
    ("SZG", "LOWS"),
    ("TLL", "EETN"),
    ("TLV", "LLBG"),
    ("TPA", "KTPA"),
    ("TPE", "RCTP"),
    ("TSN", "ZBTJ"),
    ("TUN", "DTTA"),
    ("TXL", "EDDT"),
    ("VCE", "LIPZ"),
    ("VIE", "LOWW"),
    ("WAW", "EPWA"),
    ("YEG", "CYEG"),
    ("YHZ", "CYHZ"),
    ("YOW", "CYOW"),
    ("YUL", "CYUL"),
    ("YVR", "CYVR"),
    ("YYC", "CYYC"),
    ("YYZ", "CYYZ"),
    ("ZRH", "LSZH"),
];

#[must_use]
pub fn iata_to_icao_callsign(flight_iata: &str) -> String {
    let normalized = flight_iata.trim().to_ascii_uppercase();
    if normalized.len() <= 2 {
        return normalized;
    }

    let airline_code: String = normalized.chars().take(2).collect();
    let suffix: String = normalized.chars().skip(2).collect();
    let prefix = AIRLINE_IATA_TO_ICAO
        .iter()
        .find_map(|(iata, icao)| (*iata == airline_code).then_some(*icao))
        .unwrap_or(&airline_code);

    format!("{prefix}{suffix}")
}

#[must_use]
pub fn iata_to_icao_airport(iata: &str) -> Option<&'static str> {
    let key = iata.trim().to_ascii_uppercase();
    AIRPORT_IATA_TO_ICAO
        .iter()
        .find_map(|(iata_code, icao_code)| (*iata_code == key).then_some(*icao_code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn start_mock_server(
        token_requests: Arc<AtomicUsize>,
        flights_all_body: Arc<RwLock<String>>,
        max_requests: usize,
    ) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");

        tokio::spawn(async move {
            for _ in 0..max_requests {
                let (mut stream, _) = listener.accept().await.expect("accept");
                let mut buffer = vec![0_u8; 8192];
                let read = stream.read(&mut buffer).await.expect("read request");
                let request = String::from_utf8_lossy(&buffer[..read]).to_string();

                let response = if request.starts_with("POST /token") {
                    token_requests.fetch_add(1, Ordering::Relaxed);
                    let body = r#"{"access_token":"token-abc","expires_in":1800}"#;
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                } else if request.starts_with("GET /flights/all") {
                    let body = flights_all_body.read().await.clone();
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                } else if request.starts_with("GET /flights/aircraft") {
                    let body = r#"[{"icao24":"abc123","firstSeen":100,"estDepartureAirport":"EIDW","lastSeen":200,"estArrivalAirport":"EGLL","callsign":"EIN123","estDepartureAirportHorizDistance":null,"estDepartureAirportVertDistance":null,"estArrivalAirportHorizDistance":null,"estArrivalAirportVertDistance":null,"departureAirportCandidatesCount":1,"arrivalAirportCandidatesCount":1}]"#;
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                } else {
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string()
                };

                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write response");
            }
        });

        format!("http://{addr}")
    }

    #[tokio::test]
    async fn test_token_acquisition() {
        let token_requests = Arc::new(AtomicUsize::new(0));
        let flights_all_body = Arc::new(RwLock::new("[]".to_string()));
        let base_url = start_mock_server(token_requests.clone(), flights_all_body, 3).await;
        let client = OpenSkyClient::with_urls(
            "id".to_string(),
            "secret".to_string(),
            reqwest::Client::new(),
            base_url.clone(),
            format!("{base_url}/token"),
        );

        let first = client.get_flights_all(1, 2).await;
        let second = client.get_flights_all(3, 4).await;

        assert!(first.is_ok());
        assert!(second.is_ok());
        assert_eq!(token_requests.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let token_requests = Arc::new(AtomicUsize::new(0));
        let flights_all_body = Arc::new(RwLock::new("[]".to_string()));
        let base_url = start_mock_server(token_requests.clone(), flights_all_body, 4).await;
        let client = OpenSkyClient::with_urls(
            "id".to_string(),
            "secret".to_string(),
            reqwest::Client::new(),
            base_url.clone(),
            format!("{base_url}/token"),
        );

        client
            .get_flights_all(1, 2)
            .await
            .expect("first request should work");

        {
            let mut write_guard = client.token.write().await;
            *write_guard = Some(TokenState {
                access_token: "expired-token".to_string(),
                expires_at: std::time::Instant::now(),
            });
        }

        client
            .get_flights_all(3, 4)
            .await
            .expect("second request should refresh token");

        assert_eq!(token_requests.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_flights_all_parsing() {
        let token_requests = Arc::new(AtomicUsize::new(0));
        let flights_all_body = Arc::new(RwLock::new(
            r#"[{"icao24":"abc123","firstSeen":1000,"estDepartureAirport":"EIDW","lastSeen":2000,"estArrivalAirport":"EGLL","callsign":"EIN123","estDepartureAirportHorizDistance":3,"estDepartureAirportVertDistance":4,"estArrivalAirportHorizDistance":5,"estArrivalAirportVertDistance":6,"departureAirportCandidatesCount":1,"arrivalAirportCandidatesCount":1}]"#.to_string(),
        ));
        let base_url = start_mock_server(token_requests, flights_all_body, 2).await;
        let client = OpenSkyClient::with_urls(
            "id".to_string(),
            "secret".to_string(),
            reqwest::Client::new(),
            base_url.clone(),
            format!("{base_url}/token"),
        );

        let flights = client
            .get_flights_all(1, 100)
            .await
            .expect("flights/all should parse");

        assert_eq!(flights.len(), 1);
        assert_eq!(flights[0].icao24, "abc123");
        assert_eq!(flights[0].callsign.as_deref(), Some("EIN123"));
        assert_eq!(flights[0].est_departure_airport.as_deref(), Some("EIDW"));
        assert_eq!(flights[0].est_arrival_airport.as_deref(), Some("EGLL"));
    }

    #[tokio::test]
    async fn test_verify_route_found() {
        let token_requests = Arc::new(AtomicUsize::new(0));
        let flights_all_body = Arc::new(RwLock::new(
            r#"[{"icao24":"abc123","firstSeen":1741998000,"estDepartureAirport":"EIDW","lastSeen":1742005200,"estArrivalAirport":"EGLL","callsign":"EIN123","estDepartureAirportHorizDistance":null,"estDepartureAirportVertDistance":null,"estArrivalAirportHorizDistance":null,"estArrivalAirportVertDistance":null,"departureAirportCandidatesCount":1,"arrivalAirportCandidatesCount":1}]"#.to_string(),
        ));
        let base_url = start_mock_server(token_requests, flights_all_body, 2).await;
        let client = OpenSkyClient::with_urls(
            "id".to_string(),
            "secret".to_string(),
            reqwest::Client::new(),
            base_url.clone(),
            format!("{base_url}/token"),
        );

        let verification = client
            .verify_route("EI123", "2025-03-15", "DUB", "LHR")
            .await
            .expect("verify_route should succeed")
            .expect("matching flight should be found");

        assert!(verification.operated);
        assert_eq!(verification.est_departure_airport.as_deref(), Some("EIDW"));
        assert_eq!(verification.est_arrival_airport.as_deref(), Some("EGLL"));
        assert_eq!(verification.callsign.as_deref(), Some("EIN123"));
    }

    #[tokio::test]
    async fn test_verify_route_not_found() {
        let token_requests = Arc::new(AtomicUsize::new(0));
        let flights_all_body = Arc::new(RwLock::new(
            r#"[{"icao24":"abc123","firstSeen":1741998000,"estDepartureAirport":"EIDW","lastSeen":1742005200,"estArrivalAirport":"EGLL","callsign":"BAW321","estDepartureAirportHorizDistance":null,"estDepartureAirportVertDistance":null,"estArrivalAirportHorizDistance":null,"estArrivalAirportVertDistance":null,"departureAirportCandidatesCount":1,"arrivalAirportCandidatesCount":1}]"#.to_string(),
        ));
        let base_url = start_mock_server(token_requests, flights_all_body, 13).await;
        let client = OpenSkyClient::with_urls(
            "id".to_string(),
            "secret".to_string(),
            reqwest::Client::new(),
            base_url.clone(),
            format!("{base_url}/token"),
        );

        let verification = client
            .verify_route("EI123", "2025-03-15", "DUB", "LHR")
            .await
            .expect("verify_route should succeed when no match is found");

        assert!(verification.is_none());
    }

    #[test]
    fn test_iata_to_icao_callsign() {
        assert_eq!(iata_to_icao_callsign("EI123"), "EIN123");
        assert_eq!(iata_to_icao_callsign("BA456"), "BAW456");
        assert_eq!(iata_to_icao_callsign("U2123"), "EZY123");
        assert_eq!(iata_to_icao_callsign("ZZ999"), "ZZ999");
    }

    #[test]
    fn test_iata_to_icao_airport() {
        assert_eq!(iata_to_icao_airport("DUB"), Some("EIDW"));
        assert_eq!(iata_to_icao_airport("LHR"), Some("EGLL"));
        assert_eq!(iata_to_icao_airport("JFK"), Some("KJFK"));
        assert_eq!(iata_to_icao_airport("XYZ"), None);
    }
}
