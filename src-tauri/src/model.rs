use {serde::Serialize, serde_json::json};

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

pub enum ScreenAction {
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
        presentor: User,
        title: String,
    },
}

pub fn serialize(action: ScreenAction) -> String {
    use ScreenAction::*;
    let json = match action {
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

        PresentationUpdate { presentor, title } => json!({
            "type": "presentation.update",
            "args": {
                "new": {
                    "presentor": {
                        "userIcon": presentor.icon,
                        "identifier": presentor.ident,
                        "name": presentor.name
                    },
                    "title": title
                }
            }
        }),
    };

    serde_json::to_string(&json).unwrap()
}

use diesel::Queryable;

#[derive(Queryable)]
pub struct Presentation {
    pub id: i32,
    pub title: String,
    pub presentor_id: i32,
}

#[derive(Queryable)]
pub struct Presentor {
    pub id: i32,
    pub display_name: String,
    pub twitter_id: String,
    pub icon: String,
}
