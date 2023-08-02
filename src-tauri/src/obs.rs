use {
    anyhow::{Context as _, Result},
    obws::Client,
    tokio::sync::mpsc::Receiver,
};

pub(crate) enum ObsAction {
    Mute,
    UnMute,
}

pub(crate) struct ObsClient {
    client: Client,
}

impl ObsClient {
    pub(crate) async fn connect(addr: &str, port: u16, pass: &str) -> Result<Self> {
        let client = Client::connect(addr, port, Some(pass))
            .await
            .context("failed to connect to obs client")?;

        Ok(Self { client })
    }

    pub(crate) async fn start(self, mut re: Receiver<ObsAction>) -> Result<()> {
        while let Some(action) = re.recv().await {
            let source_name_list = self
                .client
                .inputs()
                .list(None)
                .await
                .context("failed to fetch sources")?
                .into_iter()
                .map(|x| x.name);

            match action {
                ObsAction::Mute => {
                    for name in source_name_list {
                        self.client
                            .inputs()
                            .set_muted(&name, true)
                            .await
                            .with_context(|| format!("failed to mute {}", name))?;
                    }
                }

                ObsAction::UnMute => {
                    for name in source_name_list {
                        self.client
                            .inputs()
                            .set_muted(&name, false)
                            .await
                            .with_context(|| format!("failed to mute {}", name))?;
                    }
                }
            }
        }

        Ok(())
    }
}
