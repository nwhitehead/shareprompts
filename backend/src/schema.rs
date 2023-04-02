// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Text,
        title -> Varchar,
        contents -> Text,
        public -> Bool,
        research -> Bool,
        creationdate -> Timestamp,
    }
}
