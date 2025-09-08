use chrono::NaiveDateTime;
use diesel::{Associations, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::*;
use crate::user::user_model::User;

#[derive(Insertable, Queryable, Selectable, Serialize, Debug)]
#[diesel(table_name = organizations)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,

    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable, Associations, Insertable, Debug)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(User))]
#[diesel(table_name = user_organizations)]
pub struct UserOrganization {
    pub user_id: Uuid,
    pub organization_id: Uuid,

    pub role: OrganizationRole,
    added_by: Option<Uuid>,
    added_at: NaiveDateTime,
}

#[derive(diesel_derive_enum::DbEnum, PartialEq, Deserialize, Serialize, Debug)]
#[db_enum(existing_type_path = "crate::schema::sql_types::OrganizationRole")]
pub enum OrganizationRole {
    OWNER,
    ADMIN,
    EDITOR,
    COMMENTER,
    VIEWER
}

impl Organization {
    pub fn new(name: String, created_by: Uuid) -> Organization {
        Organization {
            id: Uuid::now_v7(),
            name,
            created_by,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: None,
        }
    }
}

impl UserOrganization {
    pub fn new(organization_id: Uuid, user_id: Uuid, role: OrganizationRole, added_by: Option<Uuid>) -> UserOrganization {
        UserOrganization {
            user_id,
            organization_id,
            role,
            added_by,
            added_at: chrono::Utc::now().naive_utc(),
}}
    }