use serde::Deserialize;

pub enum Outcome {
    Success,
    GiveUp,
    Failed,
}

pub trait Achievable {
    type Description: Deserialize;

    async fn attempt(&mut self) -> Outcome;
}
