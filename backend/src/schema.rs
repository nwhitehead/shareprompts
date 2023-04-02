// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Text,
        title -> Varchar,
        body -> Text,
        public -> Bool,
        research -> Bool,
    }
}
