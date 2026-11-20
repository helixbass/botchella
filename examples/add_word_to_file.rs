use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;

use botchella::{Achievable, Outcome};

#[derive(sqlx::Type)]
#[sqlx(type_name = "achievable_types", rename_all = "snake_case")]
enum AchievableType {
    AddWordToFile,
}

struct AddWordToFile {
    pub file: PathBuf,
}

impl AddWordToFile {
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }
}

#[async_trait]
impl Achievable<AchievableType> for AddWordToFile {
    type Description = Description;

    async fn attempt(&mut self, description: &Description) -> Outcome {
        let Ok(mut file) = File::options().append(true).create(true).open(&self.file) else {
            return Outcome::Failed;
        };
        let Ok(()) = file.lock() else {
            return Outcome::Failed;
        };
        let Ok(()) = writeln!(&mut file, "{}", description.word) else {
            return Outcome::Failed;
        };
        Outcome::Success
    }

    fn type_(&self) -> AchievableType {
        AchievableType::AddWordToFile
    }
}

#[derive(Deserialize)]
struct Description {
    pub word: String,
}

#[tokio::main]
async fn main() {
    let mut add_word_to_file = AddWordToFile::new("./tmp-words.txt".into());
    let description = Description {
        word: "whee".to_owned(),
    };
    let outcome = add_word_to_file.attempt(&description).await;
    println!("{outcome:#?}");

    run_achievable_if_any_outstanding();
}
