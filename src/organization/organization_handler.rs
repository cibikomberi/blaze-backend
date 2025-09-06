use actix_web::{get, post, web::{Json, Query, ServiceConfig}, HttpMessage, HttpRequest, Responder};
use actix_web::web::Path;
use uuid::Uuid;
use crate::{error::ApiResponse, organization::{organization_dto::{CreateOrganizationDTO, SearchDto}, organization_service}, user::user_model::User};
use crate::organization::organization_dto::{AddUserDTO, OrganizationIdDto};
use crate::organization::organization_model::Organization;

#[post[""]]
pub async fn create(dto: Json<CreateOrganizationDTO>, request: HttpRequest) -> Result<impl Responder, ApiResponse> {
    let CreateOrganizationDTO { name } = dto.into_inner();
    let org = organization_service::create(name, &request.extensions().get::<User>().unwrap()).await?;
    Ok(Json(org))
}

#[get("")]
pub async fn list_organizations(dto: Query<SearchDto>, request: HttpRequest) -> Result<impl Responder, ApiResponse> {
    let SearchDto { keyword, limit, cursor } = dto.into_inner();
    let org = organization_service::list_organizations(request.extensions().get::<User>().unwrap(), keyword, limit, cursor).await?;

    Ok(Json(org))
}

#[get("{organization_id}")]
pub async fn get_organization(dto: Path<OrganizationIdDto>, request: HttpRequest) -> Result<Json<Organization   >, ApiResponse> {
    let OrganizationIdDto { organization_id} = dto.into_inner();
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();

    let organization = organization_service::fetch_organization(organization_id, user).await?;
    Ok(Json(organization))
}

#[get("users/{organization_id}")]
async fn list_users_in_organization(dto: Query<SearchDto>, organization_id: Path<Uuid>, request: HttpRequest) -> Result<Json<Vec<User>>, ApiResponse> {
    let SearchDto { keyword, limit, cursor } = dto.into_inner();
    let organization_id = organization_id.into_inner();
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();
    let users = organization_service::list_users_in_organization(organization_id, keyword, limit, cursor, user).await?;
    Ok(Json(users))
}

#[post("user")]
async fn add_user(dto: Json<AddUserDTO>, request: HttpRequest) -> Result<Json<User>, ApiResponse> {
    let AddUserDTO { user_id, organization_id, role } = dto.into_inner();
    let extensions = request.extensions();
    let user = extensions.get::<User>().unwrap();
    let added_user = organization_service::add_user(user_id, organization_id, role, user).await?;
    Ok(Json(added_user))
}

pub fn organization_routes(cfg: &mut ServiceConfig) {
    cfg.service(create);
    cfg.service(get_organization);
    cfg.service(list_organizations);
    cfg.service(list_users_in_organization);
    cfg.service(add_user);
}