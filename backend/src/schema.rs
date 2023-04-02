// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Text,
        title -> Varchar,
        body -> Text,
        public -> Bool,
        research -> Bool,
    }
}
