use async_trait::async_trait;
use serde::Deserialize;

pub enum Outcome {
    Success,
    GiveUp,
    Failed,
}

#[async_trait]
pub trait Achievable {
    type Description: for<'a> Deserialize<'a>;

    async fn attempt(&mut self, description: &Self::Description) -> Outcome;
}
