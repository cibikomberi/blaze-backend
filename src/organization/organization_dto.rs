use serde::Deserialize;
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
pub struct OrganizationIdDto {
    pub organization_id: Uuid
}