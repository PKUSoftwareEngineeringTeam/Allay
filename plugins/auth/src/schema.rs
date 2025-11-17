// @generated automatically by Diesel CLI.

diesel::table! {
    sessions (token) {
        token -> Text,
        user_id -> Integer,
        expires_at -> Timestamp,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(sessions, users,);
