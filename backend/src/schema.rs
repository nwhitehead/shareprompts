// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Text,
        openaiid -> Text,
        contents -> Text,
        metadata -> Text,
        public -> Bool,
        research -> Bool,
        deleted -> Bool,
        user_id -> Text,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Text,
        conversation_count -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    conversations,
    users,
);
