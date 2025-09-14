use crate::error::ApiResponse;
use crate::file::file_dto::{FileDto, FileIdDto, FileNameDTO, FileQueryDto, SearchFileDto};
use crate::file::file_model::File;
use crate::file::file_service;
use crate::user::user_model::User;
use actix_web::web::{Bytes, Json, Path, PayloadConfig, Query, ServiceConfig};
use actix_web::{delete, get, put, web, HttpMessage, HttpRequest, Responder};
use uuid::Uuid;

#[put("{folder_id}")]
async fn upload(body: web::Bytes, folder_id: Path<Uuid>, file_name: Query<FileNameDTO>, request: HttpRequest) -> Result<(), ApiResponse> {
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();
    file_service::upload(body, folder_id.into_inner(), file_name.into_inner().file_name, user).await?;

    Ok(())
}

#[get("")]
async fn search_file(query: Query<SearchFileDto>, request: HttpRequest) -> Result<Json<Vec<File>>, ApiResponse> {
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();
    let SearchFileDto { folder_id, keyword, limit, cursor } = query.into_inner();
    let files = file_service::search_file(folder_id, keyword, limit, cursor, user).await?;
    
    Ok(Json(files))
}

#[get("{file_id}")]
async fn get_file(dto: Path<FileIdDto>, request: HttpRequest)-> Result<impl Responder, ApiResponse> {
    let extension = request.extensions();
    let user = extension.get::<User>().unwrap();
    let FileIdDto { file_id } = dto.into_inner();
    let file = file_service::get_file(file_id, user.id).await?;
    Ok(file)
}

#[delete("{file_id}")]
async fn delete_file(dto: Path<FileIdDto>, request: HttpRequest)-> Result<(), ApiResponse> {
    let extension = request.extensions();
    let user = extension.get::<User>().unwrap();
    let FileIdDto { file_id } = dto.into_inner();
    
    file_service::delete_file(file_id, user.id).await
}

#[get("{organization_name}/{bucket_name}/{file_path:.*}")]
pub async fn serve_file(dto: Path<FileDto>, query: Query<FileQueryDto>) -> Result<actix_files::NamedFile, ApiResponse> {
    let FileDto { organization_name, bucket_name, file_path } = dto.into_inner();
    file_service::serve_file(organization_name, bucket_name, file_path, query.into_inner()).await
}

#[put("{organization_name}/{bucket_name}/{file_path:.*}")]
pub async fn save_file(bytes: Bytes, dto: Path<FileDto>, query: Query<FileQueryDto>) -> Result<(), ApiResponse> {
    let FileDto { organization_name, bucket_name, file_path } = dto.into_inner();
    file_service::save_file(bytes, organization_name, bucket_name, file_path, query.into_inner()).await
}

#[delete("{organization_name}/{bucket_name}/{file_path:.*}")]
pub async fn remove_file(dto: Path<FileDto>, query: Query<FileQueryDto>) -> Result<(), ApiResponse> {
    let FileDto { organization_name, bucket_name, file_path } = dto.into_inner();
    file_service::remove_file(organization_name, bucket_name, file_path, query.into_inner()).await
}

pub fn file_routes(cfg: &mut ServiceConfig) {
    cfg.app_data(PayloadConfig::new(1 * 1024 * 1024 * 1024)).service(upload);
    cfg.service(search_file);
    cfg.service(delete_file);
    cfg.service(get_file);
}

pub fn fs_routes(cfg: &mut ServiceConfig) {
    cfg.service(serve_file);
    cfg.service(remove_file);
    cfg.app_data(PayloadConfig::new(1 * 1024 * 1024 * 1024)).service(save_file);
}