use async_trait::async_trait;
use serde::Deserialize;
use sqlx::{PgPool, Postgres};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Success,
    GiveUp,
    Failed,
}

#[async_trait]
pub trait Achievable<TTypes: sqlx::Type<Postgres>> {
    type Description: for<'a> Deserialize<'a>;

    async fn attempt(&mut self, description: &Self::Description) -> Outcome;
    fn type_(&self) -> TTypes;
}

async fn run_achievable_if_any_outstanding<
    TTypes: sqlx::Type<Postgres>,
    TDescription: for<'a> Deserialize<'a>,
>(
    pool: &PgPool,
    achievable: &mut dyn Achievable<TTypes, Description = TDescription>,
) {
    unimplemented!()
}
