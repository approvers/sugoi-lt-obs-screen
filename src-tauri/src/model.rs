use {
    crate::Context,
    serde::{Deserialize, Serialize},
    serde_json::json,
    std::sync::Arc,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub icon: Option<String>,
    pub ident: Option<String>,
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Discord,
    Twitter,
    Youtube,
}

#[derive(Serialize)]
pub enum Page {
    LTScreen,
    WaitingScreen,
}

pub(crate) enum ScreenAction {
    TimelineClear,
    TimelinePush {
        user: User,
        service: Service,
        content: String,
    },
    NotificationUpdate {
        text: String,
    },
    PresentationUpdate {
        presenter: User,
        title: String,
    },
    SwitchPage(Page),
    UpcomingPresentationsUpdate(Arc<Context>),
}

impl ScreenAction {
    pub(crate) async fn serialize(self) -> String {
        use ScreenAction::*;
        let json = match self {
            TimelineClear => json!({ "type": "timeline.flush" }),

            TimelinePush {
                user,
                service,
                content,
            } => json!({
                "type": "timeline.add",
                "args": {
                    "new": {
                        "user": {
                            "userIcon": user.icon,
                            "identifier": user.ident,
                            "name": user.name
                        },
                        "service": service,
                        "content": content,
                    }
                }
            }),

            NotificationUpdate { text } => json!({
                "type": "notification.update",
                "args": {
                    "new": text
                }
            }),

            PresentationUpdate { presenter, title } => json!({
                "type": "presentation.update",
                "args": {
                    "new": {
                        "presenter": {
                            "userIcon": presenter.icon,
                            "identifier": presenter.ident,
                            "name": presenter.name
                        },
                        "title": title
                    }
                }
            }),

            SwitchPage(page) => json!({
                "type": "screen.update",
                "args": {
                    "new": page
                }
            }),

            UpcomingPresentationsUpdate(ctx) => json!({
                "type": "waiting.pending.update",
                "args": {
                    "new": ctx.presentations.read().await.to_json_value()
                }
            }),
        };

        serde_json::to_string(&json).unwrap()
    }
}
