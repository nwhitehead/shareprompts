// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Text,
        title -> Varchar,
        contents -> Text,
        model -> Text,
        public -> Bool,
        research -> Bool,
        creationdate -> Timestamp,
        user_id -> Text,
    }
}
