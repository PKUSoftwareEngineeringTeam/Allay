// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (token) {
        token -> Text,
        user_id -> Text,
        expires_at -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(sessions, user,);
