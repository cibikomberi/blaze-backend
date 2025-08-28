use chrono::NaiveDateTime;
use diesel::{Associations, Insertable, Queryable, Selectable};
use uuid::Uuid;
use crate::schema::*;
use crate::schema::sql_types::OrganizationRole;
use crate::user::user_model::User;

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = organizations)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,

    pub created_by: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable, Associations)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(User))]
#[diesel(table_name = user_organizations)]
pub struct UserOrganization {
    pub user_id: Uuid,
    pub  organization_id: Uuid,

    pub role: OrganizationRole,
    added_by: Option<Uuid>,
    added_at: NaiveDateTime,
}