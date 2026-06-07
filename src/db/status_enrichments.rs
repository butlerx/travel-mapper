use sqlx::SqlitePool;

/// Database row for flight or rail status enrichment data.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Row {
    pub id: i64,
    pub hop_id: i64,
    pub provider: String,
    pub status: String,
    pub delay_minutes: Option<i64>,
    pub dep_gate: String,
    pub dep_terminal: String,
    pub arr_gate: String,
    pub arr_terminal: String,
    pub dep_platform: String,
    pub arr_platform: String,
    /// Aircraft registration / tail number (e.g. `"EI-DEG"`), when known.
    pub aircraft_reg: String,
    pub raw_json: String,
    pub fetched_at: String,
}

impl Row {
    /// A "no-data" sentinel row (see `write_no_data_sentinel` in the worker):
    /// every meaningful field is empty. We must not treat the step from a
    /// sentinel to real data as a notify-worthy transition.
    fn is_sentinel(&self) -> bool {
        self.status.is_empty()
            && self.delay_minutes.is_none()
            && self.dep_gate.is_empty()
            && self.dep_terminal.is_empty()
            && self.arr_gate.is_empty()
            && self.arr_terminal.is_empty()
            && self.dep_platform.is_empty()
            && self.arr_platform.is_empty()
            && self.aircraft_reg.is_empty()
    }
}

/// Map a raw provider status string to a notify-worthy human label, or `None`
/// for statuses that aren't worth a notification on their own (e.g. scheduled).
fn notable_status_label(status: &str) -> Option<&'static str> {
    match status.trim().to_ascii_lowercase().as_str() {
        "active" | "en-route" | "en route" | "departed" | "in-air" | "in air" => Some("Departed"),
        "landed" | "arrived" => Some("Landed"),
        "boarding" => Some("Boarding"),
        _ => None,
    }
}

fn push_changed_field(parts: &mut Vec<String>, label: &str, next: &str, prior: &str) {
    let next = next.trim();
    if !next.is_empty() && next != prior.trim() {
        parts.push(format!("{label} {next}"));
    }
}

/// The status fields about to be written for a hop/provider, used to detect
/// notify-worthy changes against the previously stored [`Row`].
pub struct StatusSnapshot<'a> {
    pub status: &'a str,
    pub delay_minutes: Option<i64>,
    pub dep_gate: &'a str,
    pub dep_terminal: &'a str,
    pub arr_gate: &'a str,
    pub arr_terminal: &'a str,
    pub dep_platform: &'a str,
    pub arr_platform: &'a str,
}

impl StatusSnapshot<'_> {
    /// Describe the notify-worthy transition from `prior` to this snapshot as a
    /// short human string (e.g. `"Delayed 25m \u{00b7} Gate B22"`), or `None`
    /// when nothing meaningful changed or there is no meaningful prior to
    /// compare against (initial data is not a "change").
    #[must_use]
    pub fn describe_transition(&self, prior: Option<&Row>) -> Option<String> {
        let prior = prior?;
        if prior.is_sentinel() {
            return None;
        }

        let next_status = self.status.trim().to_ascii_lowercase();
        let prior_status = prior.status.trim().to_ascii_lowercase();

        let is_cancel = |s: &str| s.contains("cancel");
        let is_divert = |s: &str| s.contains("divert") || s.contains("redirect");

        // Cancellation and diversion are the most significant — report alone.
        if is_cancel(&next_status) && !is_cancel(&prior_status) {
            return Some("Cancelled".to_owned());
        }
        if is_divert(&next_status) && !is_divert(&prior_status) {
            return Some("Diverted".to_owned());
        }

        let mut parts: Vec<String> = Vec::new();

        if next_status != prior_status
            && let Some(label) = notable_status_label(&next_status)
        {
            parts.push(label.to_owned());
        }

        let next_delay = self.delay_minutes.unwrap_or(0);
        let prior_delay = prior.delay_minutes.unwrap_or(0);
        if next_delay > 0 && next_delay != prior_delay {
            if prior_delay <= 0 {
                parts.push(format!("Delayed {next_delay}m"));
            } else if next_delay > prior_delay {
                parts.push(format!("Delay increased to {next_delay}m"));
            } else {
                parts.push(format!("Delay down to {next_delay}m"));
            }
        }

        push_changed_field(&mut parts, "Gate", self.dep_gate, &prior.dep_gate);
        push_changed_field(
            &mut parts,
            "Terminal",
            self.dep_terminal,
            &prior.dep_terminal,
        );
        push_changed_field(&mut parts, "Arr. gate", self.arr_gate, &prior.arr_gate);
        push_changed_field(
            &mut parts,
            "Arr. terminal",
            self.arr_terminal,
            &prior.arr_terminal,
        );
        push_changed_field(
            &mut parts,
            "Platform",
            self.dep_platform,
            &prior.dep_platform,
        );
        push_changed_field(
            &mut parts,
            "Arr. platform",
            self.arr_platform,
            &prior.arr_platform,
        );

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" \u{00b7} "))
        }
    }
}

/// Insert or update status enrichment data.
pub struct Upsert<'a> {
    pub hop_id: i64,
    pub provider: &'a str,
    pub status: &'a str,
    pub delay_minutes: Option<i64>,
    pub dep_gate: &'a str,
    pub dep_terminal: &'a str,
    pub arr_gate: &'a str,
    pub arr_terminal: &'a str,
    pub dep_platform: &'a str,
    pub arr_platform: &'a str,
    pub aircraft_reg: &'a str,
    pub raw_json: &'a str,
}

impl Upsert<'_> {
    /// # Errors
    ///
    /// Returns an error if the upsert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO status_enrichments
                   (hop_id, provider, status, delay_minutes,
                    dep_gate, dep_terminal, arr_gate, arr_terminal,
                    dep_platform, arr_platform, aircraft_reg, raw_json)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(hop_id, provider) DO UPDATE SET
                   status = excluded.status,
                   delay_minutes = excluded.delay_minutes,
                   dep_gate = excluded.dep_gate,
                   dep_terminal = excluded.dep_terminal,
                   arr_gate = excluded.arr_gate,
                   arr_terminal = excluded.arr_terminal,
                   dep_platform = excluded.dep_platform,
                   arr_platform = excluded.arr_platform,
                   aircraft_reg = excluded.aircraft_reg,
                   raw_json = excluded.raw_json,
                   fetched_at = datetime('now')",
            self.hop_id,
            self.provider,
            self.status,
            self.delay_minutes,
            self.dep_gate,
            self.dep_terminal,
            self.arr_gate,
            self.arr_terminal,
            self.dep_platform,
            self.arr_platform,
            self.aircraft_reg,
            self.raw_json,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Fetch the most recent status enrichment for a hop.
pub struct GetByHopId {
    pub hop_id: i64,
}

impl GetByHopId {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: i64",
                   hop_id as "hop_id!: i64",
                   provider as "provider!: String",
                   status as "status!: String",
                   delay_minutes,
                   dep_gate as "dep_gate!: String",
                   dep_terminal as "dep_terminal!: String",
                   arr_gate as "arr_gate!: String",
                   arr_terminal as "arr_terminal!: String",
                   dep_platform as "dep_platform!: String",
                   arr_platform as "arr_platform!: String",
                   aircraft_reg as "aircraft_reg!: String",
                   raw_json as "raw_json!: String",
                   fetched_at as "fetched_at!: String"
               FROM status_enrichments
               WHERE hop_id = ?
               ORDER BY fetched_at DESC
               LIMIT 1"#,
            self.hop_id,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Fetch the most recent status enrichments for multiple hops.
pub struct GetByHopIds {
    pub hop_ids: Vec<i64>,
}

impl GetByHopIds {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        if self.hop_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids_json =
            serde_json::to_string(&self.hop_ids).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        sqlx::query_as!(
            Row,
            r#"SELECT
                   se.id as "id!: i64",
                   se.hop_id as "hop_id!: i64",
                   se.provider as "provider!: String",
                   se.status as "status!: String",
                   se.delay_minutes,
                   se.dep_gate as "dep_gate!: String",
                   se.dep_terminal as "dep_terminal!: String",
                   se.arr_gate as "arr_gate!: String",
                   se.arr_terminal as "arr_terminal!: String",
                   se.dep_platform as "dep_platform!: String",
                   se.arr_platform as "arr_platform!: String",
                   se.aircraft_reg as "aircraft_reg!: String",
                   se.raw_json as "raw_json!: String",
                   se.fetched_at as "fetched_at!: String"
               FROM status_enrichments se
               INNER JOIN (
                   SELECT hop_id, MAX(fetched_at) AS max_fetched
                   FROM status_enrichments
                   WHERE hop_id IN (SELECT value FROM json_each(?))
                   GROUP BY hop_id
               ) latest ON se.hop_id = latest.hop_id AND se.fetched_at = latest.max_fetched
               WHERE se.hop_id IN (SELECT value FROM json_each(?))"#,
            ids_json,
            ids_json,
        )
        .fetch_all(pool)
        .await
    }
}

/// Fetch the most recent status enrichment for a hop from a specific provider.
pub struct GetByHopIdAndProvider<'a> {
    pub hop_id: i64,
    pub provider: &'a str,
}

impl GetByHopIdAndProvider<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: i64",
                   hop_id as "hop_id!: i64",
                   provider as "provider!: String",
                   status as "status!: String",
                   delay_minutes,
                   dep_gate as "dep_gate!: String",
                   dep_terminal as "dep_terminal!: String",
                   arr_gate as "arr_gate!: String",
                   arr_terminal as "arr_terminal!: String",
                   dep_platform as "dep_platform!: String",
                   arr_platform as "arr_platform!: String",
                   aircraft_reg as "aircraft_reg!: String",
                   raw_json as "raw_json!: String",
                   fetched_at as "fetched_at!: String"
               FROM status_enrichments
               WHERE hop_id = ? AND provider = ?
               ORDER BY fetched_at DESC
               LIMIT 1"#,
            self.hop_id,
            self.provider,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Fetch status enrichments for multiple hops from a specific provider.
pub struct GetByHopIdsAndProvider<'a> {
    pub hop_ids: Vec<i64>,
    pub provider: &'a str,
}

impl GetByHopIdsAndProvider<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        if self.hop_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids_json =
            serde_json::to_string(&self.hop_ids).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        sqlx::query_as!(
            Row,
            r#"SELECT
                   se.id as "id!: i64",
                   se.hop_id as "hop_id!: i64",
                   se.provider as "provider!: String",
                   se.status as "status!: String",
                   se.delay_minutes,
                   se.dep_gate as "dep_gate!: String",
                   se.dep_terminal as "dep_terminal!: String",
                   se.arr_gate as "arr_gate!: String",
                   se.arr_terminal as "arr_terminal!: String",
                   se.dep_platform as "dep_platform!: String",
                   se.arr_platform as "arr_platform!: String",
                   se.aircraft_reg as "aircraft_reg!: String",
                   se.raw_json as "raw_json!: String",
                   se.fetched_at as "fetched_at!: String"
               FROM status_enrichments se
               WHERE se.hop_id IN (SELECT value FROM json_each(?))
                 AND se.provider = ?"#,
            ids_json,
            self.provider,
        )
        .fetch_all(pool)
        .await
    }
}

/// Fetch all status enrichments for a hop (one per provider).
pub struct GetAllByHopId {
    pub hop_id: i64,
}

impl GetAllByHopId {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: i64",
                   hop_id as "hop_id!: i64",
                   provider as "provider!: String",
                   status as "status!: String",
                   delay_minutes,
                   dep_gate as "dep_gate!: String",
                   dep_terminal as "dep_terminal!: String",
                   arr_gate as "arr_gate!: String",
                   arr_terminal as "arr_terminal!: String",
                   dep_platform as "dep_platform!: String",
                   arr_platform as "arr_platform!: String",
                   aircraft_reg as "aircraft_reg!: String",
                   raw_json as "raw_json!: String",
                   fetched_at as "fetched_at!: String"
               FROM status_enrichments
               WHERE hop_id = ?
               ORDER BY fetched_at DESC"#,
            self.hop_id,
        )
        .fetch_all(pool)
        .await
    }
}

/// Delete all status enrichments for a hop.
pub struct DeleteByHopId {
    pub hop_id: i64,
}

impl DeleteByHopId {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM status_enrichments WHERE hop_id = ?",
            self.hop_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        hops::{Create, GetAll, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    async fn insert_hop(pool: &SqlitePool, user_id: i64) -> i64 {
        let hop = sample_hop(TravelType::Air, "LHR", "JFK", "2024-05-01", "2024-05-01");
        Create {
            trip_id: "trip-enrichment",
            user_id,
            hops: &[hop],
        }
        .execute(pool)
        .await
        .expect("insert hop");
        let rows = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(pool)
        .await
        .expect("get hops");
        rows[0].id
    }

    #[tokio::test]
    async fn upsert_and_get_by_hop_id_roundtrip() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = insert_hop(&pool, user_id).await;

        Upsert {
            hop_id,
            provider: "aviationstack",
            status: "landed",
            delay_minutes: Some(15),
            dep_gate: "B22",
            dep_terminal: "5",
            arr_gate: "C10",
            arr_terminal: "1",
            dep_platform: "",
            arr_platform: "",
            aircraft_reg: "EI-DEG",
            raw_json: r#"{"test":true}"#,
        }
        .execute(&pool)
        .await
        .expect("upsert failed");

        let row = GetByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("get failed")
            .expect("should find enrichment");

        assert_eq!(row.hop_id, hop_id);
        assert_eq!(row.status, "landed");
        assert_eq!(row.delay_minutes, Some(15));
        assert_eq!(row.dep_gate, "B22");
        assert_eq!(row.arr_terminal, "1");
        assert_eq!(row.aircraft_reg, "EI-DEG");
    }

    #[tokio::test]
    async fn upsert_updates_existing_enrichment() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = insert_hop(&pool, user_id).await;

        Upsert {
            hop_id,
            provider: "aviationstack",
            status: "active",
            delay_minutes: None,
            dep_gate: "",
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            aircraft_reg: "",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("first upsert");

        Upsert {
            hop_id,
            provider: "aviationstack",
            status: "landed",
            delay_minutes: Some(30),
            dep_gate: "A1",
            dep_terminal: "2",
            arr_gate: "D5",
            arr_terminal: "3",
            dep_platform: "",
            arr_platform: "",
            aircraft_reg: "",
            raw_json: r#"{"updated":true}"#,
        }
        .execute(&pool)
        .await
        .expect("second upsert");

        let row = GetByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("get failed")
            .expect("should find enrichment");

        assert_eq!(row.status, "landed");
        assert_eq!(row.delay_minutes, Some(30));
        assert_eq!(row.dep_gate, "A1");
    }

    #[tokio::test]
    async fn get_by_hop_ids_returns_matching_enrichments() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop1 = insert_hop(&pool, user_id).await;

        let hop2_hop = sample_hop(TravelType::Air, "JFK", "LAX", "2024-06-01", "2024-06-01");
        Create {
            trip_id: "trip-enrichment-2",
            user_id,
            hops: &[hop2_hop],
        }
        .execute(&pool)
        .await
        .expect("insert hop2");
        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("get all");
        let hop2 = all.iter().find(|h| h.origin_name == "JFK").unwrap().id;

        Upsert {
            hop_id: hop1,
            provider: "aviationstack",
            status: "landed",
            delay_minutes: Some(5),
            dep_gate: "",
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            aircraft_reg: "",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("upsert 1");

        Upsert {
            hop_id: hop2,
            provider: "aviationstack",
            status: "active",
            delay_minutes: None,
            dep_gate: "",
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            aircraft_reg: "",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("upsert 2");

        let results = GetByHopIds {
            hop_ids: vec![hop1, hop2],
        }
        .execute(&pool)
        .await
        .expect("batch get failed");

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn get_by_hop_ids_empty_returns_empty() {
        let pool = test_pool().await;
        let results = GetByHopIds {
            hop_ids: Vec::new(),
        }
        .execute(&pool)
        .await
        .expect("empty batch get failed");
        assert!(results.is_empty());
    }

    fn row_with(status: &str, delay: Option<i64>, dep_gate: &str, dep_terminal: &str) -> Row {
        Row {
            id: 1,
            hop_id: 1,
            provider: "airlabs".to_owned(),
            status: status.to_owned(),
            delay_minutes: delay,
            dep_gate: dep_gate.to_owned(),
            dep_terminal: dep_terminal.to_owned(),
            arr_gate: String::new(),
            arr_terminal: String::new(),
            dep_platform: String::new(),
            arr_platform: String::new(),
            aircraft_reg: String::new(),
            raw_json: "{}".to_owned(),
            fetched_at: "2024-01-01 00:00:00".to_owned(),
        }
    }

    fn snapshot<'a>(status: &'a str, delay: Option<i64>, dep_gate: &'a str) -> StatusSnapshot<'a> {
        StatusSnapshot {
            status,
            delay_minutes: delay,
            dep_gate,
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
        }
    }

    #[test]
    fn transition_none_without_prior() {
        assert_eq!(
            snapshot("active", Some(20), "").describe_transition(None),
            None
        );
    }

    #[test]
    fn transition_none_from_sentinel() {
        let sentinel = row_with("", None, "", "");
        assert_eq!(
            snapshot("active", Some(20), "B1").describe_transition(Some(&sentinel)),
            None
        );
    }

    #[test]
    fn transition_none_when_unchanged() {
        let prior = row_with("active", Some(10), "B1", "5");
        assert_eq!(
            snapshot("active", Some(10), "B1").describe_transition(Some(&prior)),
            None
        );
    }

    #[test]
    fn transition_delay_started() {
        let prior = row_with("active", None, "", "");
        assert_eq!(
            snapshot("active", Some(25), "").describe_transition(Some(&prior)),
            Some("Delayed 25m".to_owned())
        );
    }

    #[test]
    fn transition_delay_increased() {
        let prior = row_with("active", Some(10), "", "");
        assert_eq!(
            snapshot("active", Some(30), "").describe_transition(Some(&prior)),
            Some("Delay increased to 30m".to_owned())
        );
    }

    #[test]
    fn transition_cancelled_reported_alone() {
        let prior = row_with("scheduled", Some(15), "B1", "");
        assert_eq!(
            snapshot("cancelled", Some(15), "B1").describe_transition(Some(&prior)),
            Some("Cancelled".to_owned())
        );
    }

    #[test]
    fn transition_landed() {
        let prior = row_with("active", None, "", "");
        assert_eq!(
            snapshot("landed", None, "").describe_transition(Some(&prior)),
            Some("Landed".to_owned())
        );
    }

    #[test]
    fn transition_gate_assignment() {
        let prior = row_with("scheduled", None, "", "");
        assert_eq!(
            snapshot("scheduled", None, "B22").describe_transition(Some(&prior)),
            Some("Gate B22".to_owned())
        );
    }

    #[test]
    fn transition_combines_multiple_changes() {
        let prior = row_with("scheduled", None, "", "");
        assert_eq!(
            snapshot("active", Some(20), "B22").describe_transition(Some(&prior)),
            Some("Departed \u{00b7} Delayed 20m \u{00b7} Gate B22".to_owned())
        );
    }

    #[tokio::test]
    async fn delete_by_hop_id_removes_enrichment() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = insert_hop(&pool, user_id).await;

        Upsert {
            hop_id,
            provider: "aviationstack",
            status: "landed",
            delay_minutes: None,
            dep_gate: "",
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            aircraft_reg: "",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("upsert");

        let deleted = DeleteByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("delete failed");
        assert_eq!(deleted, 1);

        let row = GetByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("get failed");
        assert!(row.is_none());
    }
}
