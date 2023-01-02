// @generated automatically by Diesel CLI.

diesel::table! {
    user (id) {
        id -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
    }
}
