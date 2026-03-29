use sqlx::SqlitePool;

/// Database row for a named trip group.
pub struct Row {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub hop_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Insert a new trip.
pub struct Create<'a> {
    pub user_id: i64,
    pub name: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r"INSERT INTO trips (user_id, name) VALUES (?, ?)",
            self.user_id,
            self.name,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// Fetch all trips for a user.
pub struct GetAll {
    pub user_id: i64,
}

impl GetAll {
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   t.id as "id!: i64",
                   t.user_id as "user_id!: i64",
                   t.name,
                   t.start_date,
                   t.end_date,
                   COUNT(h.id) as "hop_count!: i64",
                   t.created_at as "created_at!: String",
                   t.updated_at as "updated_at!: String"
               FROM trips t
               LEFT JOIN hops h ON h.user_trip_id = t.id AND h.user_id = t.user_id
               WHERE t.user_id = ?
               GROUP BY t.id, t.user_id, t.name, t.start_date, t.end_date, t.created_at, t.updated_at
               ORDER BY COALESCE(t.start_date, t.created_at) DESC, t.id DESC"#,
            self.user_id,
        )
        .fetch_all(pool)
        .await
    }
}

/// Fetch a trip by ID and user.
pub struct GetById {
    pub id: i64,
    pub user_id: i64,
}

impl GetById {
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   t.id as "id!: i64",
                   t.user_id as "user_id!: i64",
                   t.name,
                   t.start_date,
                   t.end_date,
                   COUNT(h.id) as "hop_count!: i64",
                   t.created_at as "created_at!: String",
                   t.updated_at as "updated_at!: String"
               FROM trips t
               LEFT JOIN hops h ON h.user_trip_id = t.id AND h.user_id = t.user_id
               WHERE t.id = ? AND t.user_id = ?
               GROUP BY t.id, t.user_id, t.name, t.start_date, t.end_date, t.created_at, t.updated_at"#,
            self.id,
            self.user_id,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Update a trip's name.
pub struct Update<'a> {
    pub id: i64,
    pub user_id: i64,
    pub name: &'a str,
}

impl Update<'_> {
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r"UPDATE trips
              SET name = ?, updated_at = datetime('now')
              WHERE id = ? AND user_id = ?",
            self.name,
            self.id,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

/// Delete a trip.
pub struct Delete {
    pub id: i64,
    pub user_id: i64,
}

impl Delete {
    /// # Errors
    ///
    /// Returns an error if the delete query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM trips WHERE id = ? AND user_id = ?",
            self.id,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

/// Upsert a trip imported from `TripIt`.
///
/// If a trip with the given `tripit_id` already exists for this user, update its
/// name and dates. Otherwise insert a new trip.
pub struct UpsertFromTripIt<'a> {
    pub user_id: i64,
    pub tripit_id: &'a str,
    pub name: &'a str,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
}

impl UpsertFromTripIt<'_> {
    /// # Errors
    ///
    /// Returns an error if the upsert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let existing = sqlx::query_scalar!(
            r#"SELECT id as "id!: i64"
               FROM trips
               WHERE user_id = ? AND tripit_id = ?"#,
            self.user_id,
            self.tripit_id,
        )
        .fetch_optional(pool)
        .await?;

        if let Some(trip_id) = existing {
            sqlx::query!(
                r"UPDATE trips
                  SET name = ?, start_date = ?, end_date = ?, updated_at = datetime('now')
                  WHERE id = ? AND user_id = ?",
                self.name,
                self.start_date,
                self.end_date,
                trip_id,
                self.user_id,
            )
            .execute(pool)
            .await?;
            Ok(trip_id)
        } else {
            let result = sqlx::query!(
                r"INSERT INTO trips (user_id, name, tripit_id, start_date, end_date)
                  VALUES (?, ?, ?, ?, ?)",
                self.user_id,
                self.name,
                self.tripit_id,
                self.start_date,
                self.end_date,
            )
            .execute(pool)
            .await?;
            Ok(result.last_insert_rowid())
        }
    }
}

/// Assign all hops from a `TripIt` source trip to a local trip.
///
/// Matches hops by their `trip_id` column (format `{user_id}:tripit:{tripit_id}`)
/// and sets their `user_trip_id` to the given local trip ID.
pub struct AssignHopsBySourceTrip<'a> {
    pub user_id: i64,
    pub source_trip_id: &'a str,
    pub local_trip_id: i64,
}

impl AssignHopsBySourceTrip<'_> {
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r"UPDATE hops
              SET user_trip_id = ?, updated_at = datetime('now')
              WHERE user_id = ? AND trip_id = ?",
            self.local_trip_id,
            self.user_id,
            self.source_trip_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

/// Delete trips imported from `TripIt` that are no longer present in the API.
pub struct DeleteStaleTripItTrips<'a> {
    pub user_id: i64,
    pub active_tripit_ids: &'a [String],
}

impl DeleteStaleTripItTrips<'_> {
    /// # Errors
    ///
    /// Returns an error if the delete query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        if self.active_tripit_ids.is_empty() {
            let result = sqlx::query!(
                "DELETE FROM trips WHERE user_id = ? AND tripit_id IS NOT NULL",
                self.user_id,
            )
            .execute(pool)
            .await?;
            return Ok(result.rows_affected());
        }

        let ids_json = serde_json::to_string(self.active_tripit_ids)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let result = sqlx::query!(
            r"DELETE FROM trips
              WHERE user_id = ?
                AND tripit_id IS NOT NULL
                AND tripit_id NOT IN (SELECT value FROM json_each(?))",
            self.user_id,
            ids_json,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

/// Automatically group unassigned hops into trips based on date gaps.
pub struct AutoGroup {
    pub user_id: i64,
    pub gap_days: i64,
}

struct UnassignedHop {
    id: i64,
    start_date: String,
    end_date: String,
}

struct HopGroup {
    hop_ids: Vec<i64>,
    start_date: String,
    end_date: String,
}

impl AutoGroup {
    /// # Errors
    ///
    /// Returns an error if any database operation within the grouping transaction fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let hops = sqlx::query_as!(
            UnassignedHop,
            r#"SELECT
                   id as "id!: i64",
                   start_date as "start_date!: String",
                   end_date as "end_date!: String"
               FROM hops
               WHERE user_id = ? AND user_trip_id IS NULL
               ORDER BY start_date ASC, end_date ASC, id ASC"#,
            self.user_id,
        )
        .fetch_all(pool)
        .await?;

        let groups = group_hops_by_gap(&hops, self.gap_days);
        if groups.is_empty() {
            return Ok(0);
        }

        let mut tx = pool.begin().await?;
        let mut created = 0_u64;
        for group in &groups {
            let existing_trip_id = sqlx::query_scalar!(
                r#"SELECT id as "id!: i64"
                   FROM trips
                   WHERE user_id = ?
                     AND start_date IS NOT NULL
                     AND end_date IS NOT NULL
                     AND start_date <= ?
                     AND end_date >= ?
                   ORDER BY start_date ASC
                   LIMIT 1"#,
                self.user_id,
                group.end_date,
                group.start_date,
            )
            .fetch_optional(&mut *tx)
            .await?;

            let trip_id = if let Some(id) = existing_trip_id {
                id
            } else {
                let name = trip_name_for_start_date(&group.start_date);
                let insert = sqlx::query!(
                    r"INSERT INTO trips (user_id, name, start_date, end_date)
                      VALUES (?, ?, ?, ?)",
                    self.user_id,
                    name,
                    group.start_date,
                    group.end_date,
                )
                .execute(&mut *tx)
                .await?;
                created += 1;
                insert.last_insert_rowid()
            };

            for hop_id in &group.hop_ids {
                sqlx::query!(
                    "UPDATE hops SET user_trip_id = ? WHERE id = ? AND user_id = ?",
                    trip_id,
                    hop_id,
                    self.user_id,
                )
                .execute(&mut *tx)
                .await?;
            }

            refresh_trip_dates(&mut tx, self.user_id, trip_id).await?;
        }
        tx.commit().await?;

        Ok(created)
    }
}

/// Assign a hop to a trip.
pub struct AssignHop {
    pub hop_id: i64,
    pub trip_id: i64,
    pub user_id: i64,
}

impl AssignHop {
    /// # Errors
    ///
    /// Returns an error if any database operation within the assignment transaction fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let trip_exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                   SELECT 1 FROM trips WHERE id = ? AND user_id = ?
               ) as "exists!: i64""#,
            self.trip_id,
            self.user_id,
        )
        .fetch_one(&mut *tx)
        .await?;
        if trip_exists == 0 {
            return Ok(false);
        }

        let updated = sqlx::query!(
            r"UPDATE hops
              SET user_trip_id = ?, updated_at = datetime('now')
              WHERE id = ? AND user_id = ?",
            self.trip_id,
            self.hop_id,
            self.user_id,
        )
        .execute(&mut *tx)
        .await?;

        if updated.rows_affected() == 0 {
            return Ok(false);
        }

        refresh_trip_dates(&mut tx, self.user_id, self.trip_id).await?;
        tx.commit().await?;

        Ok(true)
    }
}

/// Unassign a hop from its trip.
pub struct UnassignHop {
    pub hop_id: i64,
    pub user_id: i64,
}

impl UnassignHop {
    /// # Errors
    ///
    /// Returns an error if any database operation within the unassignment transaction fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let mut tx = pool.begin().await?;

        let current_trip_id = sqlx::query_scalar!(
            r#"SELECT user_trip_id as "user_trip_id: i64"
               FROM hops
               WHERE id = ? AND user_id = ?"#,
            self.hop_id,
            self.user_id,
        )
        .fetch_optional(&mut *tx)
        .await?
        .flatten();

        let updated = sqlx::query!(
            r"UPDATE hops
              SET user_trip_id = NULL, updated_at = datetime('now')
              WHERE id = ? AND user_id = ?",
            self.hop_id,
            self.user_id,
        )
        .execute(&mut *tx)
        .await?;

        if updated.rows_affected() == 0 {
            return Ok(false);
        }

        if let Some(trip_id) = current_trip_id {
            refresh_trip_dates(&mut tx, self.user_id, trip_id).await?;
        }
        tx.commit().await?;

        Ok(true)
    }
}

async fn refresh_trip_dates(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    user_id: i64,
    trip_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"UPDATE trips
          SET start_date = (
                  SELECT MIN(start_date)
                  FROM hops
                  WHERE user_id = ? AND user_trip_id = ?
              ),
              end_date = (
                  SELECT MAX(end_date)
                  FROM hops
                  WHERE user_id = ? AND user_trip_id = ?
              ),
              updated_at = datetime('now')
          WHERE id = ? AND user_id = ?",
        user_id,
        trip_id,
        user_id,
        trip_id,
        trip_id,
        user_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn group_hops_by_gap(hops: &[UnassignedHop], gap_days: i64) -> Vec<HopGroup> {
    if hops.is_empty() {
        return Vec::new();
    }

    let non_negative_gap = gap_days.max(0);
    let mut groups = Vec::new();

    let mut current_ids = Vec::new();
    let mut current_start = hops[0].start_date.clone();
    let mut current_end = hops[0].end_date.clone();
    let mut last_ordinal = date_ordinal(&hops[0].end_date);

    current_ids.push(hops[0].id);

    for hop in hops.iter().skip(1) {
        let start_ordinal = date_ordinal(&hop.start_date);
        let should_split = match (last_ordinal, start_ordinal) {
            (Some(prev), Some(next)) => next - prev > non_negative_gap,
            _ => true,
        };

        if should_split {
            groups.push(HopGroup {
                hop_ids: current_ids,
                start_date: std::mem::replace(&mut current_start, hop.start_date.clone()),
                end_date: std::mem::replace(&mut current_end, hop.end_date.clone()),
            });
            current_ids = Vec::new();
            last_ordinal = date_ordinal(&hop.end_date);
        } else {
            if hop.end_date > current_end {
                current_end.clone_from(&hop.end_date);
            }
            last_ordinal = date_ordinal(&current_end);
        }

        current_ids.push(hop.id);
    }

    groups.push(HopGroup {
        hop_ids: current_ids,
        start_date: current_start,
        end_date: current_end,
    });

    groups
}

fn trip_name_for_start_date(date: &str) -> String {
    if let Some((year, month, _)) = parse_ymd(date) {
        let month_name = match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => return "Trip".to_string(),
        };
        format!("{month_name} {year} Trip")
    } else {
        "Trip".to_string()
    }
}

fn date_ordinal(date: &str) -> Option<i64> {
    let (year, month, day) = parse_ymd(date)?;
    let m = if month <= 2 { month + 12 } else { month };
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m - 3) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe)
}

fn parse_ymd(date: &str) -> Option<(i64, i64, i64)> {
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some((year, month, day))
}

#[cfg(test)]
mod tests {
    use super::{AssignHop, AutoGroup, Create, Delete, GetAll, GetById, UnassignHop, Update};
    use crate::db::{
        hops::{Create as CreateHops, GetAll as GetAllHops, Row as HopRow, TravelType},
        tests::{test_pool, test_user},
    };

    fn hop(origin: &str, dest: &str, start_date: &str) -> HopRow {
        HopRow {
            id: 0,
            travel_type: TravelType::Air,
            origin_name: origin.to_string(),
            origin_lat: 1.0,
            origin_lng: 2.0,
            origin_country: None,
            dest_name: dest.to_string(),
            dest_lat: 3.0,
            dest_lng: 4.0,
            dest_country: None,
            start_date: start_date.to_string(),
            end_date: start_date.to_string(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: None,
            cost_currency: None,
            loyalty_program: None,
            miles_earned: None,
        }
    }

    async fn hop_ids_for_user(pool: &SqlitePool, user_id: i64) -> Vec<i64> {
        let hops = GetAllHops {
            user_id,
            travel_type_filter: None,
        }
        .execute(pool)
        .await
        .expect("fetch hops failed");
        hops.into_iter().map(|h| h.id).collect()
    }

    use sqlx::SqlitePool;

    #[tokio::test]
    async fn create_get_update_delete_trip_roundtrip() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let trip_id = Create {
            user_id,
            name: "Europe 2024",
        }
        .execute(&pool)
        .await
        .expect("create trip failed");

        let found = GetById {
            id: trip_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("get by id failed")
        .expect("trip missing");
        assert_eq!(found.name, "Europe 2024");

        let updated = Update {
            id: trip_id,
            user_id,
            name: "Europe 2024 Updated",
        }
        .execute(&pool)
        .await
        .expect("update failed");
        assert!(updated);

        let renamed = GetById {
            id: trip_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("get by id failed")
        .expect("trip missing");
        assert_eq!(renamed.name, "Europe 2024 Updated");

        let deleted = Delete {
            id: trip_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("delete failed");
        assert!(deleted);
        let missing = GetById {
            id: trip_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("get by id failed");
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn get_all_includes_hop_counts() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        CreateHops {
            trip_id: "seed",
            user_id,
            hops: &[hop("DUB", "LHR", "2024-03-01")],
        }
        .execute(&pool)
        .await
        .expect("insert hop failed");
        let hop_id = hop_ids_for_user(&pool, user_id).await[0];

        let trip_id = Create {
            user_id,
            name: "UK Trip",
        }
        .execute(&pool)
        .await
        .expect("create trip failed");

        let assigned = AssignHop {
            hop_id,
            trip_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("assign hop failed");
        assert!(assigned);

        let all = GetAll { user_id }
            .execute(&pool)
            .await
            .expect("get all failed");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].hop_count, 1);
    }

    #[tokio::test]
    async fn auto_group_clusters_unassigned_hops() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        CreateHops {
            trip_id: "seed-1",
            user_id,
            hops: &[
                hop("DUB", "LHR", "2024-03-01"),
                hop("LHR", "CDG", "2024-03-02"),
                hop("CDG", "MAD", "2024-03-10"),
            ],
        }
        .execute(&pool)
        .await
        .expect("insert hops failed");

        let created = AutoGroup {
            user_id,
            gap_days: 3,
        }
        .execute(&pool)
        .await
        .expect("auto group failed");
        assert_eq!(created, 2);

        let trips = GetAll { user_id }
            .execute(&pool)
            .await
            .expect("get all failed");
        assert_eq!(trips.len(), 2);
        let total_hops: i64 = trips.iter().map(|t| t.hop_count).sum();
        assert_eq!(total_hops, 3);
    }

    #[tokio::test]
    async fn assign_and_unassign_hop_updates_trip_date_envelope() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        CreateHops {
            trip_id: "seed-2",
            user_id,
            hops: &[
                hop("DUB", "LHR", "2024-04-01"),
                hop("LHR", "JFK", "2024-04-10"),
            ],
        }
        .execute(&pool)
        .await
        .expect("insert hops failed");
        let hop_ids = hop_ids_for_user(&pool, user_id).await;

        let trip_id = Create {
            user_id,
            name: "April Trip",
        }
        .execute(&pool)
        .await
        .expect("create trip failed");

        for hop_id in &hop_ids {
            let assigned = AssignHop {
                hop_id: *hop_id,
                trip_id,
                user_id,
            }
            .execute(&pool)
            .await
            .expect("assign failed");
            assert!(assigned);
        }

        let after_assign = GetById {
            id: trip_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("get trip failed")
        .expect("trip missing");
        assert_eq!(after_assign.start_date.as_deref(), Some("2024-04-01"));
        assert_eq!(after_assign.end_date.as_deref(), Some("2024-04-10"));

        let unassigned = UnassignHop {
            hop_id: hop_ids[0],
            user_id,
        }
        .execute(&pool)
        .await
        .expect("unassign failed");
        assert!(unassigned);

        let after_unassign = GetById {
            id: trip_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("get trip failed")
        .expect("trip missing");
        assert_eq!(after_unassign.hop_count, 1);
        assert_eq!(after_unassign.start_date.as_deref(), Some("2024-04-10"));
        assert_eq!(after_unassign.end_date.as_deref(), Some("2024-04-10"));
    }
}
