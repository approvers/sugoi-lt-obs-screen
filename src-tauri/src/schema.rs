table! {
    presentations (presentation_id) {
        presentation_id -> Integer,
        title -> Text,
        presentor_id -> Nullable<Integer>,
    }
}

table! {
    presentors (presentor_id) {
        presentor_id -> Integer,
        display_name -> Text,
        twitter_id -> Text,
        icon -> Text,
    }
}

joinable!(presentations -> presentors (presentor_id));

allow_tables_to_appear_in_same_query!(
    presentations,
    presentors,
);
