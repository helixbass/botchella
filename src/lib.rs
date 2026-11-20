use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, PgPool, Postgres, Transaction};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Success,
    GiveUp,
    Failed,
}

#[async_trait]
pub trait Achievable<TTypes: sqlx::Type<Postgres>> {
    type Description: Serialize + for<'a> Deserialize<'a>;

    async fn attempt(&mut self, description: &Self::Description) -> Outcome;
    fn type_(&self) -> TTypes;
}

#[derive(sqlx::FromRow)]
struct OutstandingRow<TDescription: Serialize + for<'d> Deserialize<'d> + Send + Unpin + 'static> {
    pub achievable_id: i64,
    #[sqlx(json)]
    pub description: TDescription,
    pub action: AchievableAction,
}

async fn run_achievable_if_any_outstanding<
    TTypes: sqlx::Type<Postgres> + for<'a> sqlx::Encode<'a, Postgres>,
    TDescription: Serialize + for<'a> Deserialize<'a> + Send + Unpin + 'static,
>(
    achievable: &mut dyn Achievable<TTypes, Description = TDescription>,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    let Some(row) = fetch_outstanding_row(achievable, pool).await? else {
        return Ok(());
    };

    let mut transaction = pool.begin().await?;

    sqlx::query("LOCK TABLE achievable_attempts IN EXCLUSIVE MODE")
        .execute(&mut *transaction)
        .await?;

    if !double_check_outstanding_row(achievable, row.achievable_id, pool).await? {
        return Ok(());
    }

    record_start(row.achievable_id, &mut transaction).await?;

    Ok(())
}

async fn fetch_outstanding_row<
    TTypes: sqlx::Type<Postgres> + for<'a> sqlx::Encode<'a, Postgres>,
    TDescription: Serialize + for<'a> Deserialize<'a> + Send + Unpin + 'static,
>(
    achievable: &dyn Achievable<TTypes, Description = TDescription>,
    pool: &PgPool,
) -> Result<Option<OutstandingRow<TDescription>>, sqlx::Error> {
    sqlx::query_as::<_, OutstandingRow<TDescription>>(
        "
        SELECT
            achievables.id as achievable_id,
            achievables.description,
            most_recent_achievable_attempts.action
        FROM achievables LEFT JOIN (
            SELECT DISTINCT ON (achievable_id)
                achievable_id,
                action
            FROM achievable_attempts
            ORDER BY id DESC
        ) most_recent_achievable_attempts
        ON achievables.id = most_recent_achievable_attempts.achievable_id
        WHERE
            achievables.type = $1
            AND (
                most_recent_achievable_attempts.action IS NULL
                OR most_recent_achievable_attempts.action = 'failed'
            )
        LIMIT 1
    ",
    )
    .bind(achievable.type_())
    .fetch_optional(pool)
    .await
}

async fn double_check_outstanding_row<
    TTypes: sqlx::Type<Postgres> + for<'a> sqlx::Encode<'a, Postgres>,
    TDescription: Serialize + for<'a> Deserialize<'a> + Send + Unpin + 'static,
>(
    achievable: &dyn Achievable<TTypes, Description = TDescription>,
    achievable_id: i64,
    pool: &PgPool,
) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query(
        "
        SELECT achievables.id
        FROM achievables LEFT JOIN (
            SELECT DISTINCT ON (achievable_id)
                achievable_id,
                action
            FROM achievable_attempts
            ORDER BY id DESC
        ) most_recent_achievable_attempts
        ON achievables.id = most_recent_achievable_attempts.achievable_id
        WHERE
            achievables.type = $1
            AND (
                most_recent_achievable_attempts.action IS NULL
                OR most_recent_achievable_attempts.action = 'failed'
            )
            AND achievables.id = $2
        LIMIT 1
    ",
    )
    .bind(achievable.type_())
    .bind(achievable_id)
    .fetch_optional(pool)
    .await?
    .is_some())
}

async fn create_achievable<
    TTypes: sqlx::Type<Postgres> + for<'a> sqlx::Encode<'a, Postgres>,
    TDescription: Serialize + for<'a> Deserialize<'a>,
>(
    achievable: &dyn Achievable<TTypes, Description = TDescription>,
    description: &TDescription,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO achievables (type, description) VALUES ($1, $2)")
        .bind(achievable.type_())
        .bind(Json(description))
        .execute(pool)
        .await?;
    Ok(())
}

async fn record_start(
    achievable_id: i64,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO achievable_attempts (achievable_id, action) VALUES ($1, $2)")
        .bind(achievable_id)
        .bind(AchievableAction::Start)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn record_outcome(
    achievable_id: i64,
    outcome: Outcome,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO achievable_attempts (achievable_id, action) VALUES ($1, $2)")
        .bind(achievable_id)
        .bind(AchievableAction::from(outcome))
        .execute(pool)
        .await?;
    Ok(())
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "achievable_action", rename_all = "snake_case")]
enum AchievableAction {
    Start,
    Success,
    GiveUp,
    Failed,
}

impl From<Outcome> for AchievableAction {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Success => Self::Success,
            Outcome::GiveUp => Self::GiveUp,
            Outcome::Failed => Self::Failed,
        }
    }
}
