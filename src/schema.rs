// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Nullable<Varchar>,
        #[max_length = 6]
        locale -> Nullable<Varchar>,
    }
}
