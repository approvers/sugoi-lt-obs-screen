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
        let client = Client::connect(addr, port)
            .await
            .context("failed to connect to obs client")?;

        client
            .login(Some(pass))
            .await
            .context("failed to login to obs")?;

        Ok(Self { client })
    }

    pub(crate) async fn start(self, mut re: Receiver<ObsAction>) -> Result<()> {
        while let Some(action) = re.recv().await {
            let source_name_list = self
                .client
                .sources()
                .get_sources_list()
                .await
                .context("failed to fetch sources")?
                .into_iter()
                .map(|x| x.name);

            match action {
                ObsAction::Mute => {
                    for name in source_name_list {
                        self.client
                            .sources()
                            .set_mute(&name, true)
                            .await
                            .with_context(|| format!("failed to mute {}", name))?;
                    }
                }

                ObsAction::UnMute => {
                    for name in source_name_list {
                        self.client
                            .sources()
                            .set_mute(&name, false)
                            .await
                            .with_context(|| format!("failed to mute {}", name))?;
                    }
                }
            }
        }

        Ok(())
    }
}
