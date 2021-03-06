use std::iter::FromIterator;

use {
    crate::model::User,
    anyhow::{Context as _, Result},
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::{collections::VecDeque, path::Path},
    tokio::fs,
};

#[derive(Serialize, Deserialize)]
pub(crate) struct Presentation {
    pub(crate) presenter: User,
    pub(crate) title: String,
}

pub(crate) struct Presentations {
    list: VecDeque<Presentation>,
}

impl Presentations {
    pub(crate) fn new() -> Self {
        Self {
            list: VecDeque::new(),
        }
    }

    pub(crate) async fn save(&self, path: &Path) -> Result<()> {
        let yaml = serde_yaml::to_string(&self.list).context("failed to serialize list")?;

        fs::write(path, &yaml)
            .await
            .context("failed to write file")?;

        Ok(())
    }

    pub(crate) async fn load_from_file(path: &Path) -> Result<Self> {
        let yaml = fs::read_to_string(path)
            .await
            .context("failed to read file")?;

        let list =
            serde_yaml::from_str(&yaml).context("failed to deserialize presentation list")?;

        Ok(Self { list })
    }

    pub(crate) fn list(&self) -> String {
        self.list
            .iter()
            .enumerate()
            .map(|(n, x)| format!("{}: name: {} title: {}", n, x.title, x.presenter.name))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(crate) async fn pop(&mut self) -> bool {
        let result = self.list.pop_front().is_some();

        if result {
            self.save(Path::new("./temp_presentations.yaml"))
                .await
                .unwrap();
        }

        result
    }

    pub(crate) async fn remove(&mut self, index: usize) -> bool {
        let result = self.list.remove(index).is_some();

        if result {
            self.save(Path::new("./temp_presentations.yaml"))
                .await
                .unwrap();
        }

        result
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut Presentation> {
        self.list.get_mut(index)
    }

    pub(crate) async fn push(&mut self, p: Presentation) {
        self.list.push_back(p);

        self.save(Path::new("./temp_presentations.yaml"))
            .await
            .unwrap();
    }

    pub(crate) fn to_json_value(&self) -> Value {
        Value::from_iter(
            self.list.iter().map(|x| {
                serde_json::to_value(x).expect("failed to serialize presentation to json")
            }),
        )
    }
}
