use {
    crate::model::User,
    anyhow::{Context as _, Result},
    serde::{Deserialize, Serialize},
    std::path::Path,
    tokio::fs,
};

#[derive(Serialize, Deserialize)]
pub(crate) struct Presentation {
    presentor: User,
    title: String,
}

pub(crate) struct Presentations {
    list: Vec<Presentation>,
}

impl Presentations {
    pub(crate) async fn load_from_file(path: &Path) -> Result<Self> {
        let yaml = fs::read_to_string(path)
            .await
            .context("failed to read file")?;

        let list =
            serde_yaml::from_str(&yaml).context("failed to deserialize presentation list")?;

        Ok(Self { list })
    }
}
