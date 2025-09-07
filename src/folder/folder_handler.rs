use actix_web::{delete, get, post, HttpMessage, HttpRequest, Responder};
use actix_web::web::{Json, Path, Query, ServiceConfig};
use crate::error::ApiResponse;
use crate::file::file_service;
use crate::folder::folder_dto::{CreateFolderDTO, FolderIdDto, FolderResponseDto, SearchFolderDto};
use crate::folder::folder_model::Folder;
use crate::folder::folder_service;
use crate::user::user_model::User;

#[post("")]
pub async fn create(dto: Json<CreateFolderDTO>, request: HttpRequest) -> Result<Json<Folder>, ApiResponse> {
    let CreateFolderDTO { name, bucket_id, parent_id } = dto.into_inner();
    let folder = folder_service::create(name, bucket_id, parent_id, request.extensions().get::<User>().unwrap()).await?;
    Ok(Json(folder))
}

#[get("")]
async fn get(dto: Query<SearchFolderDto>, request: HttpRequest) -> Result<impl Responder, ApiResponse> {
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();
    let SearchFolderDto { bucket_id, folder_id, cursor, cursor_kind, limit, keyword } = dto.into_inner();
    let (folder, items) = folder_service::find(bucket_id, folder_id, keyword, limit, cursor, cursor_kind, user).await?;

    Ok(Json(FolderResponseDto { folder, items }))
}

#[delete("{folder_id}")]
async fn delete_folder(dto: Path<FolderIdDto>, request: HttpRequest) -> Result<(), ApiResponse> {
    let extension = request.extensions();
    let user = extension.get::<User>().unwrap();
    let FolderIdDto { folder_id } = dto.into_inner();

    folder_service::delete_folder(folder_id, user.id).await
}

pub fn folder_routes(cfg: &mut ServiceConfig) {
    cfg.service(create);
    cfg.service(delete_folder);
    cfg.service(get);
}