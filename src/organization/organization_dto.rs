use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator_derive::Validate;
use crate::organization::organization_model::OrganizationRole;

#[derive(Deserialize, Validate)]
pub struct CreateOrganizationDTO {
    #[validate(length(min = 4, max = 255))]
    pub name: String
}

#[derive(Deserialize)]
pub struct SearchDto {
    pub keyword: Option<String>,
    pub limit: i64,
    pub cursor: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct AddUserDTO {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub role: OrganizationRole
}
#[derive(Deserialize)]
pub struct DeleteUserDTO {
    pub user_id: Uuid,
    pub organization_id: Uuid,
}

#[derive(Deserialize)]
pub struct OrganizationIdDto {
    pub organization_id: Uuid
}

#[derive(Serialize)]
pub struct UserDto {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct OrganizationUserRoleDto {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub username: String,
    pub role: OrganizationRole,
    pub added_at: NaiveDateTime,
    pub added_by: Option<UserDto>
}

#[derive(Deserialize)]
pub struct PaginatedSecretSearchDto {
    pub organization_id: Uuid,
    pub limit: Option<i64>,
    pub page: Option<i64>
}

#[derive(Deserialize)]
pub struct DeleteSecretDto {
    pub organization_id: Uuid,
    pub id: String
}

#[derive(Deserialize)]
pub struct SignatureDto {
    pub id: String,
    pub signature: String,
}