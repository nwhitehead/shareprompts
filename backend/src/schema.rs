// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Text,
        contents -> Text,
        metadata -> Text,
        public -> Bool,
        research -> Bool,
        deleted -> Bool,
        user_id -> Text,
    }
}
