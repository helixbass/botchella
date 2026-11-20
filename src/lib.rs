use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, PgPool, Postgres};

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

async fn run_achievable_if_any_outstanding<
    TTypes: sqlx::Type<Postgres>,
    TDescription: Serialize + for<'a> Deserialize<'a>,
>(
    achievable: &mut dyn Achievable<TTypes, Description = TDescription>,
    pool: &PgPool,
) {
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
