# Rust Database Query Style (sqlx + SQLite)

Conventions for a Rust database layer built on `sqlx`. Follow these when
adding or changing any query. The general rules in
[rust-style.md](rust-style.md) still apply; this page adds what is specific
to queries.

Stack: `sqlx` with the `sqlite` feature, compile-time checked macros,
migrations via `sqlx migrate`.

## Core rules

1. **One query = one struct.** Inputs are fields; the query runs in
   `execute(&self, pool)`.
2. **Compile-time checked SQL only**: `sqlx::query!`, `query_as!`,
   `query_scalar!`. No runtime `sqlx::query()` except PRAGMAs.
3. **DB functions return `Result<T, sqlx::Error>`.** No `anyhow`. Propagate
   with `?`.
4. **Every query is scoped by owner** (`WHERE ... AND user_id = ?`) when the
   table has an owner.
5. **Regenerate the offline query cache** (`cargo sqlx prepare`) after any SQL
   change and commit `.sqlx/`.
6. **Never edit a committed migration.** Add a new one.

## Module layout

```text
src/
  db.rs                 # pool setup, ID newtypes, `pub mod` per table, test helpers
  db/
    users.rs            # small table: all query structs in one file
    projects.rs
    tasks.rs            # large table: shared types + re-exports
    tasks/
      create.rs         # one file per query struct
      get_all.rs
      get_by_id.rs
      update_by_id.rs
      delete_for_project.rs
```

- One module per table. No `mod.rs`; use `foo.rs` + `foo/`.
- Every `pub mod` gets a doc comment naming the table and its purpose:

  ```rust
  /// Query objects for the `tasks` table — individual work items.
  pub mod tasks;
  ```

- Small tables keep every query struct in one file.
- When a table grows past roughly 6–8 queries, split it: each query gets its
  own private submodule, and the parent re-exports them so callers still write
  `db::tasks::Create`:

  ```rust
  mod create;
  mod get_all;
  mod get_by_id;

  pub use create::Create;
  pub use get_all::GetAll;
  pub use get_by_id::{DetailRow, GetById};
  ```

- Shared row types, enums and `pub(super)` helpers (e.g. inserting child rows
  inside a transaction) live in the parent file (`tasks.rs`). No `helpers.rs`
  / `types.rs`.

## Query objects

Each query is a struct named with a short verb-noun. The struct holds the
parameters; `execute` runs it.

```rust
use super::{ProjectId, UserId};
use sqlx::SqlitePool;

/// Insert a new project.
pub struct Create<'a> {
    pub user_id: UserId,
    pub name: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<ProjectId, sqlx::Error> {
        let result = sqlx::query!(
            r"INSERT INTO projects (user_id, name) VALUES (?, ?)",
            self.user_id.0,
            self.name,
        )
        .execute(pool)
        .await?;
        Ok(ProjectId(result.last_insert_rowid()))
    }
}
```

### Full CRUD example (`db/projects.rs`)

The rest of the file, below `Create`:

```rust
/// Database row for a project.
pub struct Row {
    pub id: ProjectId,
    pub user_id: UserId,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Fetch all projects for a user, newest first.
pub struct GetAll {
    pub user_id: UserId,
}

impl GetAll {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: ProjectId",
                   user_id as "user_id!: UserId",
                   name,
                   created_at as "created_at!: String",
                   updated_at as "updated_at!: String"
               FROM projects
               WHERE user_id = ?
               ORDER BY created_at DESC, id DESC"#,
            self.user_id.0,
        )
        .fetch_all(pool)
        .await
    }
}

/// Fetch one project by ID, scoped to its owner.
pub struct GetById {
    pub id: ProjectId,
    pub user_id: UserId,
}

impl GetById {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: ProjectId",
                   user_id as "user_id!: UserId",
                   name,
                   created_at as "created_at!: String",
                   updated_at as "updated_at!: String"
               FROM projects
               WHERE id = ? AND user_id = ?"#,
            self.id.0,
            self.user_id.0,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Rename a project.
pub struct Update<'a> {
    pub id: ProjectId,
    pub user_id: UserId,
    pub name: &'a str,
}

impl Update<'_> {
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r"UPDATE projects
              SET name = ?, updated_at = datetime('now')
              WHERE id = ? AND user_id = ?",
            self.name,
            self.id.0,
            self.user_id.0,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

/// Delete a project.
pub struct Delete {
    pub id: ProjectId,
    pub user_id: UserId,
}

impl Delete {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM projects WHERE id = ? AND user_id = ?",
            self.id.0,
            self.user_id.0,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
```

Call sites in a handler:

```rust
use crate::db::projects;

let id = projects::Create { user_id, name: &form.name }
    .execute(&state.pool)
    .await?;

let all = projects::GetAll { user_id }.execute(&state.pool).await?;

let Some(project) = projects::GetById { id, user_id }.execute(&state.pool).await? else {
    return StatusCode::NOT_FOUND.into_response();
};

if !(projects::Update { id, user_id, name: &form.name }).execute(&state.pool).await? {
    return StatusCode::NOT_FOUND.into_response();
}

if !(projects::Delete { id, user_id }).execute(&state.pool).await? {
    return StatusCode::NOT_FOUND.into_response();
}
```

### Naming

| Pattern                                             | Use for                               |
| --------------------------------------------------- | ------------------------------------- |
| `Create`, `CreateFromCsv`, `CreateManual`           | Inserts (suffix describes the source) |
| `GetAll`, `GetById`, `GetByUserId`, `GetForProject` | Reads                                 |
| `Update`, `UpdateById`                              | Updates                               |
| `Delete`, `DeleteForProject`, `DeleteStale`         | Deletes                               |
| `Upsert…`, `GetOrCreate`                            | Insert-or-update                      |
| `ExistsIn…`, `Count…`                               | Scalar checks                         |
| `Search`                                            | Multi-filter reads                    |

The table module carries the noun: `db::projects::GetAll`, never
`GetAllProjects`.

### Struct rules

- Borrow inputs: `&'a str`, `&'a [T]`, `Option<&'a str>`. Use `impl Foo<'_>`
  on the impl block.
- IDs are newtypes, never bare `i64`; see [IDs](#ids).
- Doc comment on the struct says what the query does in one line.
- Every `execute` has a `/// # Errors` section (required by
  `clippy::pedantic`).

### IDs

Each table's primary key gets a newtype in `db.rs`, so one table's ID can't
be passed where another's is expected. `GetById { id: user_id, user_id: id }`
is a compile error, not a query that quietly returns nothing.

```rust
// db.rs
/// A row in `projects`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct ProjectId(i64);

/// A row in `users`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct UserId(i64);
```

- The field is private. Rust privacy reaches descendants, so every `db::*`
  query module can read and build it (`self.id.0`, `ProjectId(rowid)`), but
  code outside `db` can't make up an ID: it gets one from a query, or from a
  request through `serde` (`Path<ProjectId>`), which is then checked by an
  owner-scoped query.
- Read with a column override: `id as "id!: ProjectId"`. Foreign keys take
  the referenced table's type: `tasks.project_id` is `"project_id!: ProjectId"`.
- Bind with `.0` (`self.id.0`): the `query!` macros check bind parameters
  against the column's type, which is `i64`.
- No type aliases (`type ProjectId = i64`): an alias is the same type as
  `i64`, so it documents but doesn't check.

### Return types

| Query kind                 | Return                          | Fetch method                                           |
| -------------------------- | ------------------------------- | ------------------------------------------------------ |
| Insert                     | The table's ID (`ProjectId(last_insert_rowid())`) | `.execute()`                         |
| Update / delete one row    | `bool` (`rows_affected() > 0`)  | `.execute()`                                           |
| Bulk insert / delete       | `u64` (count)                   | `.execute()`                                           |
| Single row, may be missing | `Option<Row>`                   | `.fetch_optional()`                                    |
| List                       | `Vec<Row>`                      | `.fetch_all()`                                         |
| Scalar                     | `i64` / `Option<i64>` / `bool`  | `query_scalar!` + `.fetch_one()` / `.fetch_optional()` |

Return `bool` from update/delete so handlers can map `false` to 404 without a
second query.

## Writing SQL

- Raw string literals: `r"..."` for plain SQL, `r#"..."#` when the SQL has `"`
  (column type overrides).
- Multi-line SELECTs: one column per line, aligned under `SELECT`.
- Use table aliases (`t`, `p`) on any query with a join.
- Positional `?` params by default. Use numbered `?1`, `?2` when a param
  repeats.
- Use SQLite's `datetime('now')` for `updated_at` in the SQL, not a Rust
  timestamp.
- Always include a deterministic `ORDER BY` on list queries (add `id` as
  tiebreaker).

```rust
r#"SELECT
       p.id as "id!: ProjectId",
       p.user_id as "user_id!: UserId",
       p.name,
       COUNT(t.id) as "task_count!: i64",
       GROUP_CONCAT(DISTINCT t.status) as "statuses?: String",
       p.created_at as "created_at!: String"
   FROM projects p
   LEFT JOIN tasks t ON t.project_id = p.id AND t.user_id = p.user_id
   WHERE p.user_id = ?
   GROUP BY p.id, p.user_id, p.name, p.created_at
   ORDER BY p.created_at DESC, p.id DESC"#
```

### Column type overrides

SQLite's type inference is weak, especially for joins, aggregates and
`COALESCE`. Override explicitly:

| Override                | Meaning                             |
| ----------------------- | ----------------------------------- |
| `col as "col!: i64"`    | Force non-null with this type       |
| `id as "id!: ProjectId"` | Force non-null, read as an ID newtype |
| `col as "col?: String"` | Force nullable                      |
| `col as "col: String"`  | Set type, keep inferred nullability |

Always override: `id` and foreign-key columns (to their ID newtype), `COUNT(...)`, `EXISTS(...)`, `COALESCE(...)`,
`GROUP_CONCAT(...)`, and any column from a `LEFT JOIN`.

### Optional filters

Use the `?N IS NULL OR` pattern with numbered params instead of building SQL
at runtime:

```rust
let name_pattern = self.name.map(|s| format!("%{s}%"));

sqlx::query_as!(
    TaskRow,
    r#"SELECT ...
       FROM tasks t
       WHERE t.user_id = ?1
         AND (?2 IS NULL OR t.status = ?2)
         AND (?3 IS NULL OR t.name LIKE ?3)
         AND (?4 IS NULL OR t.due_date >= ?4)
       ORDER BY t.due_date ASC"#,
    self.user_id.0,
    self.status,
    name_pattern,
    self.due_after,
)
```

Prepare derived values (LIKE patterns, lowercased strings) as locals before
the macro.

If one optional filter changes the shape of the query a lot, a `match` with
two separate `query_as!` calls is acceptable.

### Dynamic IN clauses

No `QueryBuilder`. Serialize the list to JSON and use `json_each()`:

```rust
let ids_json = serde_json::to_string(&self.ids)
    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

sqlx::query!(
    "DELETE FROM tasks
     WHERE user_id = ?
       AND external_id NOT IN (SELECT value FROM json_each(?))",
    self.user_id.0,
    ids_json,
)
```

Handle the empty-list case explicitly before this, when it should behave
differently.

### Upserts

Prefer `INSERT ... ON CONFLICT(key) DO UPDATE SET ...` / `DO NOTHING`. Use
`INSERT OR REPLACE` only when replacing the whole row (it deletes and
reinserts, which changes the rowid and fires cascades).

### Existence checks

```rust
let exists = sqlx::query_scalar!(
    r#"SELECT EXISTS(
           SELECT 1 FROM projects WHERE id = ? AND user_id = ?
       ) as "exists!: i64""#,
    self.project_id.0,
    self.user_id.0,
)
.fetch_one(pool)
.await?;
Ok(exists != 0)
```

## Row mapping

Two patterns:

1. **Simple rows**: `query_as!` straight into the public `Row` struct.
2. **Rows needing conversion** (enums stored as text, derived fields): query
   into a private `FooRow` that mirrors the SQL output exactly, then
   `impl TryFrom<FooRow> for Foo`.

```rust
/// Internal row type for `query_as!` (`status` is stored as text).
pub(super) struct TaskRow {
    pub id: TaskId,
    pub status: String,
    pub name: String,
}

impl TryFrom<TaskRow> for Task {
    type Error = sqlx::Error;

    fn try_from(row: TaskRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            status: parse_status(&row.status)?,
            name: row.name,
        })
    }
}

// In execute():
rows.into_iter().map(Task::try_from).collect()
```

Domain parse errors are `thiserror` types wrapped in `sqlx::Error::Decode`:

```rust
#[derive(Debug, thiserror::Error)]
#[error("unknown status '{value}': expected open|done")]
struct ParseStatusError {
    value: String,
}

pub(super) fn parse_status(value: &str) -> Result<Status, sqlx::Error> {
    match value {
        "open" => Ok(Status::Open),
        "done" => Ok(Status::Done),
        other => Err(sqlx::Error::Decode(Box::new(ParseStatusError {
            value: other.to_string(),
        }))),
    }
}
```

Each query may return its own row type (`Row`, `SummaryRow`, `DetailRow`)
sized to what the caller needs. Do not fetch every column for a listing.

## Transactions

Use a transaction whenever one `execute` touches more than one statement that
must succeed together.

```rust
pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let updated = sqlx::query!(
        r"UPDATE tasks
          SET project_id = ?, updated_at = datetime('now')
          WHERE id = ? AND user_id = ?",
        self.project_id.0,
        self.task_id.0,
        self.user_id.0,
    )
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Ok(false); // tx drops → rollback
    }

    refresh_project_dates(&mut tx, self.user_id, self.project_id).await?;
    tx.commit().await?;
    Ok(true)
}
```

- Queries inside use `&mut *tx`.
- Early `return` before `commit()` rolls back automatically on drop.
- Shared in-transaction helpers are `pub(super) async fn` taking
  `tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>` and run queries on
  `&mut **tx`.
- Short-circuit empty input before opening a transaction
  (`if self.items.is_empty() { return Ok(0); }`).

## Pool setup

```rust
pub async fn create_pool(database_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    let options = database_url
        .parse::<sqlx::sqlite::SqliteConnectOptions>()?
        .create_if_missing(true);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // PRAGMAs are the only allowed runtime `sqlx::query()` calls.
    sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
    sqlx::query("PRAGMA busy_timeout=5000").execute(&pool).await?;

    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}
```

## Testing

- Tests are inline: `#[cfg(test)] mod tests { ... }` at the bottom of the
  query file.
- All tests are `#[tokio::test]`.
- Each test gets a fresh in-memory DB with migrations applied. Shared helpers
  live in `db.rs`:

```rust
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    pub(crate) async fn test_pool() -> SqlitePool {
        let url = format!("sqlite:file:{}?mode=memory&cache=shared", Uuid::new_v4());
        create_pool(&url).await.expect("failed to create test pool")
    }

    pub(crate) async fn test_user(pool: &SqlitePool, username: &str) -> UserId {
        users::Create { username, password_hash: "hash" }
            .execute(pool)
            .await
            .expect("failed to create test user")
    }
}
```

- Use the query objects themselves to set up and assert state (create with
  `Create`, check with `GetAll`). Avoid raw SQL in tests.
- Name tests after behaviour: `get_all_filters_by_status`,
  `create_duplicate_key_replaces_row`, `delete_stale_removes_inactive`.
- Use `.expect("what failed")` in tests, never bare `.unwrap()`.
- Cover: the round trip, each filter, owner scoping (user A cannot see user
  B's rows), and the not-found / empty-input paths.

## Checklist for a new query

1. A new table gets an ID newtype in `db.rs`.
1. Add a struct with a verb-noun name in the table's module (or a new file
   under `db/<table>/` and re-export it).
2. Write SQL with a `query!` macro, owner scoping, explicit type overrides
   (IDs read as their newtype, bound with `.0`) and an `ORDER BY`.
3. Pick the return type from the table above.
4. Add `/// # Errors` and a one-line struct doc comment.
5. Add inline `#[tokio::test]` tests.
6. Run `cargo sqlx prepare` and commit `.sqlx/`.
