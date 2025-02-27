// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        password -> Nullable<Varchar>,
    }
}
