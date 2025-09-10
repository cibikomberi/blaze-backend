use crate::bucket::bucket_dto::{BucketIdDTO, SearchBucketDto, UpdateBucketDto};
use crate::{bucket::{bucket_dto::CreateBucketDto, bucket_model::Bucket, bucket_service}, error::ApiResponse, user::user_model::User};
use actix_web::web::{Path, Query};
use actix_web::{delete, get, post, put, web::{Json, ServiceConfig}, HttpMessage, HttpRequest};

#[post("")]
async fn create(dto: Json<CreateBucketDto>, request: HttpRequest) -> Result<Json<Bucket>, ApiResponse> {
    let CreateBucketDto { name, organization_id } = dto.into_inner();
    let bucket = bucket_service::create(name, organization_id, request.extensions().get::<User>().unwrap()).await?;
    Ok(Json(bucket))
}

#[get("")]
async fn list(dto: Query<SearchBucketDto>, request: HttpRequest) -> Result<Json<Vec<Bucket>>, ApiResponse> {
    let SearchBucketDto { organization_id, keyword, limit, cursor, } = dto.into_inner();
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();

    let buckets = bucket_service::list(organization_id, keyword, limit, cursor, user).await?;
    Ok(Json(buckets))
}

#[put("")]
async fn update(dto: Json<UpdateBucketDto>, request: HttpRequest) -> Result<Json<Bucket>, ApiResponse> {
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();
    let UpdateBucketDto { bucket_id, name, visibility} = dto.into_inner();
    let bucket = bucket_service::update_bucket(bucket_id, name, visibility, user).await?;
    Ok(Json(bucket))
}

#[delete("{bucket_id}")]
async fn delete(dto: Path<BucketIdDTO>, request: HttpRequest) -> Result<Json<Bucket>, ApiResponse> {
    let BucketIdDTO { bucket_id } = dto.into_inner();
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();

    let bucket = bucket_service::delete_bucket(bucket_id, user).await?;
    Ok(Json(bucket))
}

pub fn bucket_routes(cfg: &mut ServiceConfig) {
    cfg.service(list);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}