use super::rail_status::{RailStatus, RailStatusApi, RailStatusError, RailStatusQuery};
use crate::geocode::stations::crs_from_name;
use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::NotKeyed};
use quick_xml::{Reader, events::Event};
use serde_json::json;
use std::num::NonZeroU32;

const DARWIN_API_BASE: &str = "https://lite.realtime.nationalrail.co.uk/OpenLDBWS/ldb12.asmx";
const GET_DEPARTURE_BOARD_ACTION: &str =
    "http://thalesgroup.com/RTTI/2012-01-13/ldb/GetDepartureBoard";
const MAX_RETRIES: u32 = 3;
const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct DarwinClient {
    api_token: String,
    client: reqwest::Client,
    base_url: String,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

#[derive(Debug, Clone, Default)]
struct DarwinService {
    std: String,
    etd: String,
    platform: String,
    operator: String,
    operator_code: String,
    service_id: String,
}

impl DarwinClient {
    #[must_use]
    pub fn new(api_token: String) -> Self {
        let client = reqwest::Client::new();
        Self::with_base_url(api_token, client, DARWIN_API_BASE.to_string())
    }

    #[must_use]
    pub fn with_base_url(api_token: String, client: reqwest::Client, base_url: String) -> Self {
        let quota = Quota::per_hour(NonZeroU32::new(5_000).unwrap_or(NonZeroU32::MIN));
        Self {
            api_token,
            client,
            base_url,
            rate_limiter: RateLimiter::direct(quota),
        }
    }

    async fn post_soap(&self, soap_action: &str, body: String) -> Result<String, RailStatusError> {
        if self.rate_limiter.check().is_err() {
            return Err(RailStatusError::RateLimited);
        }

        tracing::debug!(
            url = self.base_url,
            action = soap_action,
            "Darwin SOAP request"
        );

        let mut attempt = 0;
        loop {
            attempt += 1;
            let result = self
                .client
                .post(&self.base_url)
                .header("Content-Type", "text/xml; charset=utf-8")
                .header("SOAPAction", soap_action)
                .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .body(body.clone())
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        return Err(RailStatusError::RateLimited);
                    }
                    if status.is_server_error() && attempt <= MAX_RETRIES {
                        sleep_with_backoff(attempt).await;
                        continue;
                    }
                    resp.error_for_status_ref().map_err(RailStatusError::Http)?;
                    let text = resp.text().await.map_err(RailStatusError::Http)?;
                    return Ok(text);
                }
                Err(err) => {
                    if (err.is_connect() || err.is_timeout()) && attempt <= MAX_RETRIES {
                        sleep_with_backoff(attempt).await;
                        continue;
                    }
                    return Err(RailStatusError::Http(err));
                }
            }
        }
    }
}

fn build_departure_board_request(
    api_token: &str,
    origin_crs: &str,
    dest_crs: Option<&str>,
) -> String {
    let mut filter_xml = String::new();
    if let Some(code) = dest_crs {
        filter_xml = format!("<ldb:filterCrs>{}</ldb:filterCrs>", escape_xml(code));
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"
               xmlns:typ="http://thalesgroup.com/RTTI/2013-11-28/Token/types"
               xmlns:ldb="http://thalesgroup.com/RTTI/2017-10-01/ldb/">
  <soap:Header>
    <typ:AccessToken>
      <typ:TokenValue>{}</typ:TokenValue>
    </typ:AccessToken>
  </soap:Header>
  <soap:Body>
    <ldb:GetDepartureBoardRequest>
      <ldb:numRows>10</ldb:numRows>
      <ldb:crs>{}</ldb:crs>
      {}
      <ldb:timeOffset>-30</ldb:timeOffset>
      <ldb:timeWindow>120</ldb:timeWindow>
    </ldb:GetDepartureBoardRequest>
  </soap:Body>
</soap:Envelope>"#,
        escape_xml(api_token),
        escape_xml(origin_crs),
        filter_xml,
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn sleep_duration_for_attempt(attempt: u32) -> std::time::Duration {
    std::time::Duration::from_millis(500 * u64::from(2_u32.pow(attempt - 1)))
}

async fn sleep_with_backoff(attempt: u32) {
    let delay = sleep_duration_for_attempt(attempt);
    tokio::time::sleep(delay).await;
}

fn local_name(raw_name: &[u8]) -> String {
    let full = String::from_utf8_lossy(raw_name);
    full.split(':')
        .next_back()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn decode_text_event(event: &Event<'_>) -> Result<String, RailStatusError> {
    match event {
        Event::Text(text) => text
            .unescape()
            .map(std::borrow::Cow::into_owned)
            .map_err(|e| RailStatusError::Parse(e.to_string())),
        Event::CData(text) => Ok(String::from_utf8_lossy(text.as_ref()).into_owned()),
        _ => Ok(String::new()),
    }
}

fn parse_services_or_fault(xml: &str) -> Result<Vec<DarwinService>, RailStatusError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut services = Vec::new();
    let mut current_service = DarwinService::default();
    let mut in_service = false;
    let mut current_tag = String::new();
    let mut in_fault = false;
    let mut fault_text = String::new();

    loop {
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| RailStatusError::Parse(e.to_string()))?;

        match event {
            Event::Start(ref start) => {
                let tag = local_name(start.name().as_ref());
                if tag == "fault" {
                    in_fault = true;
                    fault_text.clear();
                }
                if tag == "service" {
                    in_service = true;
                    current_service = DarwinService::default();
                }
                current_tag = tag;
            }
            Event::Text(_) | Event::CData(_) => {
                let text = decode_text_event(&event)?;
                if in_fault && (current_tag == "faultstring" || current_tag == "text") {
                    fault_text.clone_from(&text);
                }
                if in_service {
                    assign_service_field(&mut current_service, &current_tag, text);
                }
            }
            Event::End(ref end) => {
                let tag = local_name(end.name().as_ref());
                if tag == "service" {
                    in_service = false;
                    services.push(current_service.clone());
                }
                if tag == "fault" {
                    let message = if fault_text.is_empty() {
                        "SOAP fault from Darwin API".to_string()
                    } else {
                        fault_text.clone()
                    };
                    return Err(RailStatusError::Parse(message));
                }
                current_tag.clear();
            }
            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }

    Ok(services)
}

fn assign_service_field(service: &mut DarwinService, current_tag: &str, text: String) {
    match current_tag {
        "std" => service.std = text,
        "etd" => service.etd = text,
        "platform" => service.platform = text,
        "operator" => service.operator = text,
        "operatorcode" => service.operator_code = text,
        "serviceid" => service.service_id = text,
        _ => {}
    }
}

fn extract_hhmm(value: &str) -> Option<(u32, u32)> {
    let bytes = value.as_bytes();
    for window in bytes.windows(5) {
        if is_hhmm_window(window) {
            let hour = parse_two_digits(&window[0..2])?;
            let minute = parse_two_digits(&window[3..5])?;
            if hour < 24 && minute < 60 {
                return Some((hour, minute));
            }
        }
    }
    None
}

fn is_hhmm_window(window: &[u8]) -> bool {
    window[0].is_ascii_digit()
        && window[1].is_ascii_digit()
        && window[2] == b':'
        && window[3].is_ascii_digit()
        && window[4].is_ascii_digit()
}

fn parse_two_digits(bytes: &[u8]) -> Option<u32> {
    let text = std::str::from_utf8(bytes).ok()?;
    text.parse::<u32>().ok()
}

fn minutes_since_midnight(value: &str) -> Option<i64> {
    let (hour, minute) = extract_hhmm(value)?;
    Some(i64::from(hour) * 60 + i64::from(minute))
}

fn calculate_delay_minutes(std: &str, etd: &str) -> Option<i64> {
    let std_minutes = minutes_since_midnight(std)?;
    let etd_minutes = minutes_since_midnight(etd)?;
    let mut diff = etd_minutes - std_minutes;
    if diff < -720 {
        diff += 1_440;
    }
    Some(diff)
}

fn carrier_matches(service: &DarwinService, carrier: &str) -> bool {
    carrier.is_empty()
        || service.operator_code.eq_ignore_ascii_case(carrier)
        || service.operator.eq_ignore_ascii_case(carrier)
}

fn find_matching_service<'a>(
    services: &'a [DarwinService],
    query: &RailStatusQuery<'_>,
) -> Option<&'a DarwinService> {
    let query_time = minutes_since_midnight(query.start_date);

    if let Some(expected_time) = query_time {
        for service in services {
            if !carrier_matches(service, query.carrier) {
                continue;
            }
            let Some(std_time) = minutes_since_midnight(&service.std) else {
                continue;
            };
            if std_time == expected_time {
                return Some(service);
            }
        }
        return None;
    }

    services
        .iter()
        .find(|service| carrier_matches(service, query.carrier))
}

fn rail_status_from_service(service: &DarwinService) -> Result<RailStatus, RailStatusError> {
    let etd_lower = service.etd.to_ascii_lowercase();
    let (status, dep_delay_minutes) = if etd_lower == "on time" {
        ("on_time".to_string(), Some(0))
    } else if etd_lower == "cancelled" {
        ("cancelled".to_string(), None)
    } else if etd_lower == "delayed" {
        ("delayed".to_string(), None)
    } else if minutes_since_midnight(&service.etd).is_some() {
        (
            "delayed".to_string(),
            calculate_delay_minutes(&service.std, &service.etd),
        )
    } else {
        ("delayed".to_string(), None)
    };

    let raw_json = serde_json::to_string(&json!({
        "std": service.std,
        "etd": service.etd,
        "platform": service.platform,
        "operator": service.operator,
        "operatorCode": service.operator_code,
        "serviceID": service.service_id,
    }))
    .map_err(|e| RailStatusError::Parse(e.to_string()))?;

    Ok(RailStatus {
        status,
        dep_delay_minutes,
        arr_delay_minutes: None,
        dep_platform: service.platform.clone(),
        arr_platform: String::new(),
        raw_json,
    })
}

#[async_trait::async_trait]
impl RailStatusApi for DarwinClient {
    fn provider_name(&self) -> &'static str {
        "darwin"
    }

    async fn get_rail_status(
        &self,
        query: &RailStatusQuery<'_>,
    ) -> Result<Option<RailStatus>, RailStatusError> {
        let Some(origin_crs) = crs_from_name(query.origin_name) else {
            return Ok(None);
        };
        let destination_crs = crs_from_name(query.dest_name);

        let soap_request =
            build_departure_board_request(&self.api_token, origin_crs, destination_crs);
        let xml = self
            .post_soap(GET_DEPARTURE_BOARD_ACTION, soap_request)
            .await?;

        let services = parse_services_or_fault(&xml)?;
        if services.is_empty() {
            return Ok(None);
        }

        let Some(service) = find_matching_service(&services, query) else {
            return Ok(None);
        };

        rail_status_from_service(service).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    fn sample_query() -> RailStatusQuery<'static> {
        RailStatusQuery {
            carrier: "",
            train_number: "",
            origin_name: "London Paddington",
            dest_name: "Edinburgh",
            origin_country: Some("gb"),
            dest_country: Some("gb"),
            start_date: "2026-03-27 14:30",
            end_date: "2026-03-27 17:00",
            origin_lat: 51.5154,
            origin_lng: -0.1755,
            dest_lat: 55.952,
            dest_lng: -3.189,
        }
    }

    async fn spawn_mock_server(status_line: &str, content_type: &str, body: String) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 16_384];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        format!("http://127.0.0.1:{port}")
    }

    fn envelope_with_services(services_xml: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"
               xmlns:lt7="http://thalesgroup.com/RTTI/2017-10-01/ldb/types">
  <soap:Body>
    <GetDepartureBoardResponse>
      <GetStationBoardResult>
        <lt7:trainServices>
          {services_xml}
        </lt7:trainServices>
      </GetStationBoardResult>
    </GetDepartureBoardResponse>
  </soap:Body>
</soap:Envelope>"#,
        )
    }

    fn service_xml(std: &str, etd: &str, platform: Option<&str>) -> String {
        let platform_xml = platform
            .map(|value| format!("<lt7:platform>{value}</lt7:platform>"))
            .unwrap_or_default();

        format!(
            r"<lt7:service>
  <lt7:std>{std}</lt7:std>
  <lt7:etd>{etd}</lt7:etd>
  {platform_xml}
  <lt7:operator>LNER</lt7:operator>
  <lt7:operatorCode>GR</lt7:operatorCode>
  <lt7:serviceID>svc-1</lt7:serviceID>
</lt7:service>",
        )
    }

    #[tokio::test]
    async fn happy_path() {
        let body = envelope_with_services(&service_xml("14:30", "14:35", Some("3")));
        let base_url = spawn_mock_server("200 OK", "text/xml", body).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let query = sample_query();
        let status = client
            .get_rail_status(&query)
            .await
            .unwrap()
            .expect("expected status");

        assert_eq!(status.status, "delayed");
        assert_eq!(status.dep_delay_minutes, Some(5));
        assert_eq!(status.dep_platform, "3");
    }

    #[tokio::test]
    async fn on_time_response() {
        let body = envelope_with_services(&service_xml("10:00", "On time", Some("2")));
        let base_url = spawn_mock_server("200 OK", "text/xml", body).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let mut query = sample_query();
        query.start_date = "2026-03-27 10:00";

        let status = client
            .get_rail_status(&query)
            .await
            .unwrap()
            .expect("expected status");

        assert_eq!(status.status, "on_time");
        assert_eq!(status.dep_delay_minutes, Some(0));
    }

    #[tokio::test]
    async fn cancelled_response() {
        let body = envelope_with_services(&service_xml("12:15", "Cancelled", Some("4")));
        let base_url = spawn_mock_server("200 OK", "text/xml", body).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let mut query = sample_query();
        query.start_date = "2026-03-27 12:15";

        let status = client
            .get_rail_status(&query)
            .await
            .unwrap()
            .expect("expected status");

        assert_eq!(status.status, "cancelled");
        assert_eq!(status.dep_delay_minutes, None);
    }

    #[tokio::test]
    async fn no_platform() {
        let body = envelope_with_services(&service_xml("14:30", "14:35", None));
        let base_url = spawn_mock_server("200 OK", "text/xml", body).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let query = sample_query();
        let status = client
            .get_rail_status(&query)
            .await
            .unwrap()
            .expect("expected status");

        assert_eq!(status.dep_platform, "");
    }

    #[tokio::test]
    async fn crs_not_found() {
        let body = envelope_with_services(&service_xml("14:30", "14:35", Some("1")));
        let base_url = spawn_mock_server("200 OK", "text/xml", body).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let mut query = sample_query();
        query.origin_name = "Nonexistent Station 12345";
        let status = client.get_rail_status(&query).await.unwrap();
        assert!(status.is_none());
    }

    #[tokio::test]
    async fn soap_fault() {
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
  <soap:Body>
    <soap:Fault>
      <soap:Reason>
        <soap:Text xml:lang="en">Invalid token</soap:Text>
      </soap:Reason>
    </soap:Fault>
  </soap:Body>
</soap:Envelope>"#
            .to_string();
        let base_url = spawn_mock_server("200 OK", "text/xml", body).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let query = sample_query();
        let result = client.get_rail_status(&query).await;
        assert!(matches!(result, Err(RailStatusError::Parse(_))));
    }

    #[tokio::test]
    async fn rate_limited() {
        let base_url =
            spawn_mock_server("429 Too Many Requests", "text/plain", String::new()).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let query = sample_query();
        let result = client.get_rail_status(&query).await;
        assert!(matches!(result, Err(RailStatusError::RateLimited)));
    }

    #[tokio::test]
    async fn empty_train_services() {
        let body = envelope_with_services("");
        let base_url = spawn_mock_server("200 OK", "text/xml", body).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let query = sample_query();
        let status = client.get_rail_status(&query).await.unwrap();
        assert!(status.is_none());
    }

    #[tokio::test]
    async fn delayed_no_time() {
        let body = envelope_with_services(&service_xml("09:45", "Delayed", Some("7")));
        let base_url = spawn_mock_server("200 OK", "text/xml", body).await;
        let client =
            DarwinClient::with_base_url("token".to_string(), reqwest::Client::new(), base_url);

        let mut query = sample_query();
        query.start_date = "2026-03-27 09:45";

        let status = client
            .get_rail_status(&query)
            .await
            .unwrap()
            .expect("expected status");

        assert_eq!(status.status, "delayed");
        assert_eq!(status.dep_delay_minutes, None);
    }
}
