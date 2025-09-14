// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "bucket_visibility"))]
    pub struct BucketVisibility;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "organization_role"))]
    pub struct OrganizationRole;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::BucketVisibility;

    buckets (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        organization_id -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        visibility -> BucketVisibility,
    }
}

diesel::table! {
    files (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        folder_id -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    folders (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        bucket_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        created_by -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    organization_secrets (id) {
        #[max_length = 16]
        id -> Bpchar,
        #[max_length = 32]
        secret -> Bpchar,
        organization_id -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamp,
    }
}

diesel::table! {
    organizations (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        created_by -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OrganizationRole;

    user_organizations (user_id, organization_id) {
        user_id -> Uuid,
        organization_id -> Uuid,
        role -> OrganizationRole,
        added_by -> Nullable<Uuid>,
        added_at -> Timestamp,
    }
}

diesel::table! {
    user_session (id) {
        id -> Uuid,
        jti -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        password -> Nullable<Varchar>,
        is_verified -> Bool,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
        #[max_length = 511]
        image -> Nullable<Varchar>,
    }
}

diesel::joinable!(buckets -> users (created_by));
diesel::joinable!(files -> folders (folder_id));
diesel::joinable!(files -> users (created_by));
diesel::joinable!(folders -> buckets (bucket_id));
diesel::joinable!(folders -> users (created_by));
diesel::joinable!(organization_secrets -> organizations (organization_id));
diesel::joinable!(organization_secrets -> users (created_by));
diesel::joinable!(organizations -> users (created_by));
diesel::joinable!(user_organizations -> organizations (organization_id));
diesel::joinable!(user_session -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    buckets,
    files,
    folders,
    organization_secrets,
    organizations,
    user_organizations,
    user_session,
    users,
);
