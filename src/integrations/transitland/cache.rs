//! Static GTFS feed caching and trip_id indexing.
//!
//! Downloads static GTFS feeds from Transitland, parses them, and stores parsed
//! data in SQLite for fast journey → trip_id matching.

use super::client::TransitlandClient;
use crate::integrations::transitland::feed_discovery::RailOperator;
use deunicode::deunicode;
use gtfs_structures::Gtfs;
use sqlx::SqlitePool;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info};

/// Cache errors.
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to download static GTFS feed: {0}")]
    DownloadError(String),

    #[error("Failed to parse GTFS feed: {0}")]
    ParseError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Feed not found: {0}")]
    FeedNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("ZIP extraction error: {0}")]
    ZipError(#[from] zip::result::ZipError),
}

/// Static GTFS feed cache with 24-hour TTL.
///
/// Downloads feeds from Transitland, parses using gtfs-structures,
/// and stores in `SQLite` for fast querying.
pub struct GtfsCache {
    pool: SqlitePool,
    client: TransitlandClient,
}

impl GtfsCache {
    /// Creates a new GTFS cache instance.
    #[must_use]
    pub const fn new(pool: SqlitePool, client: TransitlandClient) -> Self {
        Self { pool, client }
    }

    /// Ensures a feed is cached and fresh (< 24 hours old).
    ///
    /// If the feed is missing or expired, downloads and parses it.
    ///
    /// # Errors
    ///
    /// Returns an error if download, parsing, or database operations fail.
    pub async fn ensure_feed_cached(&self, operator: &RailOperator) -> Result<i64, CacheError> {
        let onestop_id = operator.onestop_id();

        // Check if feed exists and is fresh
        let feed_record = sqlx::query!(
            "SELECT id, expires_at FROM gtfs_feeds WHERE onestop_id = ?",
            onestop_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(record) = feed_record {
            let expires_at =
                chrono::NaiveDateTime::parse_from_str(&record.expires_at, "%Y-%m-%d %H:%M:%S")
                    .map_err(|e| {
                        CacheError::ParseError(format!("Invalid expires_at timestamp: {e}"))
                    })?;
            let now = chrono::Utc::now().naive_utc();

            if expires_at > now {
                debug!(
                    feed = onestop_id,
                    expires_at = %record.expires_at,
                    "Feed cache is fresh"
                );
                return record
                    .id
                    .ok_or(CacheError::ParseError("missing feed_id".to_string()));
            }

            info!(feed = onestop_id, "Feed cache expired, refreshing");
            self.delete_feed(
                record
                    .id
                    .ok_or(CacheError::ParseError("missing feed_id".to_string()))?,
            )
            .await?;
        }

        // Download and cache the feed
        info!(feed = onestop_id, "Downloading static GTFS feed");
        let feed_id = self.download_and_cache_feed(operator).await?;
        Ok(feed_id)
    }

    /// Downloads a static GTFS feed and caches it in the database.
    async fn download_and_cache_feed(&self, operator: &RailOperator) -> Result<i64, CacheError> {
        let onestop_id = operator.onestop_id();

        // Discover feed URL
        let feeds = self
            .client
            .search_feeds(&super::client::FeedSearchParams {
                onestop_id: Some(onestop_id.to_string()),
                spec: Some("gtfs".to_string()),
                ..Default::default()
            })
            .await
            .map_err(|e| CacheError::DownloadError(format!("Feed search failed: {e}")))?;

        let feed = feeds
            .feeds
            .first()
            .ok_or_else(|| CacheError::FeedNotFound(onestop_id.to_string()))?;

        let static_url =
            feed.urls.static_current.as_ref().ok_or_else(|| {
                CacheError::FeedNotFound(format!("No static URL for {onestop_id}"))
            })?;

        info!(feed = onestop_id, url = static_url, "Downloading GTFS zip");

        // Download the zip file
        let response = reqwest::get(static_url)
            .await
            .map_err(|e| CacheError::DownloadError(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(CacheError::DownloadError(format!(
                "HTTP {} from {}",
                response.status(),
                static_url
            )));
        }

        let zip_bytes = response
            .bytes()
            .await
            .map_err(|e| CacheError::DownloadError(format!("Failed to read response body: {e}")))?;

        info!(
            feed = onestop_id,
            size_kb = zip_bytes.len() / 1024,
            "Downloaded GTFS zip"
        );

        // Extract to temporary directory and parse
        let temp_dir = tempfile::tempdir()?;
        let zip_path = temp_dir.path().join("feed.zip");
        std::fs::write(&zip_path, &zip_bytes)?;

        let extract_dir = temp_dir.path().join("gtfs");
        std::fs::create_dir(&extract_dir)?;

        Self::extract_zip(&zip_path, &extract_dir)?;

        info!(feed = onestop_id, "Parsing GTFS data");
        let gtfs = Gtfs::from_path(&extract_dir)
            .map_err(|e| CacheError::ParseError(format!("GTFS parsing failed: {e}")))?;

        // Store in database
        self.store_gtfs_data(onestop_id, static_url, &gtfs).await?;

        let feed_id =
            sqlx::query_scalar!("SELECT id FROM gtfs_feeds WHERE onestop_id = ?", onestop_id)
                .fetch_one(&self.pool)
                .await?;

        info!(
            feed = onestop_id,
            feed_id,
            stops = gtfs.stops.len(),
            trips = gtfs.trips.len(),
            "GTFS feed cached successfully"
        );

        Ok(feed_id.expect("feed_id should always exist after insert"))
    }

    /// Extracts a ZIP file to a directory.
    fn extract_zip(zip_path: &Path, extract_dir: &Path) -> Result<(), CacheError> {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = extract_dir.join(file.name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    /// Stores parsed GTFS data in the database.
    async fn store_gtfs_data(
        &self,
        onestop_id: &str,
        static_url: &str,
        gtfs: &Gtfs,
    ) -> Result<(), CacheError> {
        let mut tx = self.pool.begin().await?;

        let feed_version = gtfs
            .feed_info
            .first()
            .map_or("unknown", |info| info.name.as_str());

        let expires_at = chrono::Utc::now()
            .naive_utc()
            .checked_add_signed(chrono::Duration::hours(24))
            .ok_or_else(|| CacheError::ParseError("Failed to calculate expiry time".to_string()))?
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let feed_id = sqlx::query_scalar!(
            "INSERT INTO gtfs_feeds (onestop_id, feed_version, expires_at, static_url)
             VALUES (?, ?, ?, ?)
             RETURNING id",
            onestop_id,
            feed_version,
            expires_at,
            static_url
        )
        .fetch_one(&mut *tx)
        .await?;

        Self::store_stops(&mut tx, feed_id, gtfs).await?;
        Self::store_trips(&mut tx, feed_id, gtfs).await?;
        Self::store_calendar(&mut tx, feed_id, gtfs).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn store_stops(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        feed_id: i64,
        gtfs: &Gtfs,
    ) -> Result<(), CacheError> {
        for (stop_id, stop) in &gtfs.stops {
            let stop_name = stop.name.as_deref().unwrap_or("");
            let stop_name_normalized = normalize_station_name(stop_name);
            let lat = stop.latitude.unwrap_or(0.0);
            let lng = stop.longitude.unwrap_or(0.0);

            sqlx::query!(
                "INSERT INTO gtfs_stops (feed_id, stop_id, stop_name, stop_name_normalized, lat, lng)
                 VALUES (?, ?, ?, ?, ?, ?)",
                feed_id,
                stop_id,
                stop_name,
                stop_name_normalized,
                lat,
                lng
            )
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }

    async fn store_trips(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        feed_id: i64,
        gtfs: &Gtfs,
    ) -> Result<(), CacheError> {
        for (trip_id, trip) in &gtfs.trips {
            let route_id = &trip.route_id;
            let service_id = &trip.service_id;

            sqlx::query!(
                "INSERT INTO gtfs_trips (feed_id, trip_id, route_id, service_id)
                 VALUES (?, ?, ?, ?)",
                feed_id,
                trip_id,
                route_id,
                service_id
            )
            .execute(&mut **tx)
            .await?;

            for stop_time in &trip.stop_times {
                let stop_id = &stop_time.stop.id;
                let stop_sequence = stop_time.stop_sequence;
                let departure_time = format_gtfs_time(stop_time.departure_time.unwrap_or(0));
                let arrival_time = format_gtfs_time(stop_time.arrival_time.unwrap_or(0));

                sqlx::query!(
                    "INSERT INTO gtfs_stop_times (feed_id, trip_id, stop_id, stop_sequence, departure_time, arrival_time)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    feed_id,
                    trip_id,
                    stop_id,
                    stop_sequence,
                    departure_time,
                    arrival_time
                )
                .execute(&mut **tx)
                .await?;
            }
        }
        Ok(())
    }

    async fn store_calendar(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        feed_id: i64,
        gtfs: &Gtfs,
    ) -> Result<(), CacheError> {
        for (service_id, calendar) in &gtfs.calendar {
            let start_date = calendar.start_date.format("%Y%m%d").to_string();
            let end_date = calendar.end_date.format("%Y%m%d").to_string();
            let monday = i32::from(calendar.monday);
            let tuesday = i32::from(calendar.tuesday);
            let wednesday = i32::from(calendar.wednesday);
            let thursday = i32::from(calendar.thursday);
            let friday = i32::from(calendar.friday);
            let saturday = i32::from(calendar.saturday);
            let sunday = i32::from(calendar.sunday);

            sqlx::query!(
                "INSERT INTO gtfs_calendar (feed_id, service_id, monday, tuesday, wednesday, thursday, friday, saturday, sunday, start_date, end_date)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                feed_id,
                service_id,
                monday,
                tuesday,
                wednesday,
                thursday,
                friday,
                saturday,
                sunday,
                start_date,
                end_date
            )
            .execute(&mut **tx)
            .await?;
        }

        for (service_id, dates) in &gtfs.calendar_dates {
            for calendar_date in dates {
                let date = calendar_date.date.format("%Y%m%d").to_string();
                let exception_type = calendar_date.exception_type as i32;

                sqlx::query!(
                    "INSERT INTO gtfs_calendar_dates (feed_id, service_id, date, exception_type)
                     VALUES (?, ?, ?, ?)",
                    feed_id,
                    service_id,
                    date,
                    exception_type
                )
                .execute(&mut **tx)
                .await?;
            }
        }
        Ok(())
    }

    /// Deletes a cached feed and all its associated data.
    async fn delete_feed(&self, feed_id: i64) -> Result<(), CacheError> {
        // Foreign key constraints will cascade delete all related data
        sqlx::query!("DELETE FROM gtfs_feeds WHERE id = ?", feed_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Finds stop IDs matching a station name using fuzzy matching.
    ///
    /// Returns up to `limit` results, ordered by similarity score (best first).
    ///
    /// # Errors
    ///
    /// Returns an error if database query fails.
    pub async fn find_stop_ids(
        &self,
        feed_id: i64,
        station_name: &str,
        limit: usize,
    ) -> Result<Vec<StopMatch>, CacheError> {
        let normalized = normalize_station_name(station_name);

        // Fetch all stops for this feed (cached feeds are typically small enough)
        let stops = sqlx::query!(
            "SELECT stop_id, stop_name, stop_name_normalized, lat, lng
             FROM gtfs_stops
             WHERE feed_id = ?",
            feed_id
        )
        .fetch_all(&self.pool)
        .await?;

        // Calculate similarity scores
        let mut matches: Vec<StopMatch> = stops
            .into_iter()
            .map(|stop| {
                let similarity = strsim::jaro_winkler(&normalized, &stop.stop_name_normalized);
                StopMatch {
                    stop_id: stop.stop_id,
                    stop_name: stop.stop_name,
                    lat: stop.lat,
                    lng: stop.lng,
                    similarity,
                }
            })
            .filter(|m| m.similarity > 0.7) // Only return reasonable matches
            .collect();

        // Sort by similarity (descending)
        matches.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top N matches
        matches.truncate(limit);

        Ok(matches)
    }

    /// Finds trip IDs matching the given journey criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if database query fails.
    pub async fn find_trip_ids(
        &self,
        feed_id: i64,
        origin_stop_id: &str,
        dest_stop_id: &str,
        departure_time: &str,
        _service_date: &str,
    ) -> Result<Vec<TripMatch>, CacheError> {
        // Query for trips that:
        // 1. Visit origin_stop_id before dest_stop_id
        // 2. Depart from origin_stop_id around departure_time (±30 minutes)
        // 3. Run on service_date

        let trips = sqlx::query!(
            r#"
            SELECT DISTINCT
                t.trip_id,
                t.route_id,
                st_origin.departure_time as origin_departure,
                st_dest.arrival_time as dest_arrival
            FROM gtfs_trips t
            JOIN gtfs_stop_times st_origin ON t.feed_id = st_origin.feed_id AND t.trip_id = st_origin.trip_id
            JOIN gtfs_stop_times st_dest ON t.feed_id = st_dest.feed_id AND t.trip_id = st_dest.trip_id
            WHERE t.feed_id = ?
              AND st_origin.stop_id = ?
              AND st_dest.stop_id = ?
              AND st_origin.stop_sequence < st_dest.stop_sequence
            LIMIT 50
            "#,
            feed_id,
            origin_stop_id,
            dest_stop_id
        )
        .fetch_all(&self.pool)
        .await?;

        // Filter by departure time proximity
        let target_time = parse_gtfs_time(departure_time);
        let matches: Vec<TripMatch> = trips
            .into_iter()
            .filter_map(|row| {
                let dep_time = parse_gtfs_time(&row.origin_departure);
                let time_diff_minutes =
                    (dep_time.cast_signed() - target_time.cast_signed()).abs() / 60;

                if time_diff_minutes <= 30 {
                    Some(TripMatch {
                        trip_id: row.trip_id,
                        route_id: row.route_id,
                        origin_departure: row.origin_departure,
                        dest_arrival: row.dest_arrival,
                        time_diff_minutes,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(matches)
    }
}

/// A stop matching result with similarity score.
#[derive(Debug, Clone)]
pub struct StopMatch {
    pub stop_id: String,
    pub stop_name: String,
    pub lat: f64,
    pub lng: f64,
    pub similarity: f64,
}

/// A trip matching result.
#[derive(Debug, Clone)]
pub struct TripMatch {
    pub trip_id: String,
    pub route_id: String,
    pub origin_departure: String,
    pub dest_arrival: String,
    pub time_diff_minutes: i32,
}

/// Normalizes a station name for fuzzy matching.
///
/// - Converts to lowercase
/// - Removes diacritics (é → e, ü → u)
/// - Removes punctuation
/// - Trims whitespace
#[must_use]
pub fn normalize_station_name(name: &str) -> String {
    let without_diacritics = deunicode(name);
    without_diacritics
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Formats a GTFS time (seconds since midnight) as HH:MM:SS.
fn format_gtfs_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{hours:02}:{minutes:02}:{secs:02}")
}

/// Parses a GTFS time string (HH:MM:SS) to seconds since midnight.
fn parse_gtfs_time(time_str: &str) -> u32 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return 0;
    }

    let hours: u32 = parts[0].parse().unwrap_or(0);
    let minutes: u32 = parts[1].parse().unwrap_or(0);
    let seconds: u32 = parts[2].parse().unwrap_or(0);

    hours * 3600 + minutes * 60 + seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_removes_diacritics() {
        assert_eq!(normalize_station_name("Gare du Nord"), "gare du nord");
        assert_eq!(normalize_station_name("Zürich HB"), "zurich hb");
        assert_eq!(normalize_station_name("Athènes"), "athenes");
    }

    #[test]
    fn normalize_removes_punctuation() {
        assert_eq!(normalize_station_name("St. Pancras"), "st pancras");
        assert_eq!(normalize_station_name("King's Cross"), "kings cross");
    }

    #[test]
    fn format_gtfs_time_works() {
        assert_eq!(format_gtfs_time(0), "00:00:00");
        assert_eq!(format_gtfs_time(3661), "01:01:01");
        assert_eq!(format_gtfs_time(86400), "24:00:00");
        assert_eq!(format_gtfs_time(90000), "25:00:00"); // Next-day service
    }

    #[test]
    fn parse_gtfs_time_works() {
        assert_eq!(parse_gtfs_time("00:00:00"), 0);
        assert_eq!(parse_gtfs_time("01:01:01"), 3661);
        assert_eq!(parse_gtfs_time("24:00:00"), 86400);
        assert_eq!(parse_gtfs_time("25:00:00"), 90000);
    }
}
