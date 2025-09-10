use serde::Deserialize;
use uuid::Uuid;
use validator_derive::Validate;
use crate::bucket::bucket_model::BucketVisibility;

#[derive(Deserialize, Validate)]
pub struct CreateBucketDto {
    #[validate(length(min = 4, max = 255))]
    pub name: String,
    pub organization_id: Uuid
}

#[derive(Deserialize, Validate)]
pub struct UpdateBucketDto {
    pub bucket_id: Uuid,

    #[validate(length(min = 4, max = 255))]
    pub name: Option<String>,
    pub visibility: Option<BucketVisibility>
}

#[derive(Deserialize)]
pub struct SearchBucketDto {
    pub organization_id: Uuid,
    pub keyword: Option<String>,
    pub limit: i64,
    pub cursor: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct BucketIdDTO {
    pub bucket_id: Uuid
}