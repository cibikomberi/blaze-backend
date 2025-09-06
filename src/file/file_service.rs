use crate::bucket::bucket_model::Bucket;
use crate::config::db_config;
use crate::error::ApiResponse;
use crate::file::file_model::File;
use crate::folder::folder_model::Folder;
use crate::folder::folder_service;
use crate::folder::folder_service::folder_path;
use crate::organization::organization_model::{Organization, OrganizationRole, UserOrganization};
use crate::organization::organization_service;
use crate::schema::files;
use crate::schema::{buckets, user_organizations};
use crate::schema::{folders, organizations};
use crate::user::user_model::User;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::Responder;
use diesel::{ExpressionMethods, PgTextExpressionMethods, SelectableHelper};
use diesel::{JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

lazy_static! {
    static ref EDITABLE_ROLES: [OrganizationRole; 3] = [OrganizationRole::OWNER, OrganizationRole::ADMIN, OrganizationRole::EDITOR];
}
pub async fn upload(body: Bytes, folder_id: Uuid, file_name:String, user: &User) -> Result<(), ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let folder = folders::dsl::folders.find(folder_id)
        .first::<Folder>(&mut conn)
        .await?;

    let buc = buckets::dsl::buckets.find(folder.bucket_id)
        .first::<Bucket>(&mut conn)
        .await?;
    let (organization, user_organization) = organization_service::validate_access(buc.organization_id, user.id, &mut conn).await
        .map_err(|_| ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()))?;
    match user_organization {
        Some(user_organization) if EDITABLE_ROLES.contains(&user_organization.role) => (),
        _ => return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()))
    }
    let file = diesel::insert_into(files::table)
        .values(File::new(file_name, folder_id, user.id))
        .get_result::<File>(&mut conn)
        .await?;
    let mut path = "files/".to_owned() + &organization.name + "/" + &buc.name;
    path.push_str(&folder_path(folder.id, &mut conn).await?);
    path.push_str(file.name.as_str());
    drop(conn);

    // let path = Path::new(&path);
    let _ = fs::File::create(path).await
        .map_err(|_| ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Cannot create file".to_string()))?
        .write_all(&body).await
        .map_err(|_| ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Cannot write to file".to_string()))?;
    Ok(())
}

pub async fn search_file(folder_id: Uuid, keyword: Option<String>, limit: i64, cursor: Option<Uuid>, user: &User) -> Result<Vec<File>, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (folder, user_organization) = folders::dsl::folders.find(folder_id)
        .left_join(buckets::table)
        .left_join(user_organizations::table.on(user_organizations::organization_id.eq(buckets::organization_id)))
        .filter(user_organizations::user_id.eq(user.id))
        .select((Folder::as_select(), Option::<UserOrganization>::as_select()))
        .get_result::<(Folder, Option<UserOrganization>)>(&mut conn)
        .await?;

    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::UNAUTHORIZED, "User doesnt have access to this organization".to_string()));
    }

    let mut query = files::dsl::files
        .filter(files::folder_id.eq(folder_id))
        .order(files::id)
        .into_boxed();
    if let Some(keyword) = keyword {
        query = query.filter(files::name.ilike(format!("%{}%", keyword)));
    }
    if let Some(cursor) = cursor {
        query = query.filter(files::id.gt(cursor));
    }
    let files = query.limit(limit)
        .load::<File>(&mut conn)
        .await?;
    Ok(files)
}

pub async fn get_file(file_id: Uuid, user_id: Uuid) -> Result<impl Responder, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    error!("{}", file_id);
    let (file, folder, bucket, organization, user_org) = files::table.find(file_id)
        .left_join(folders::table.on(folders::id.eq(files::folder_id)))
        .left_join(buckets::table.on(buckets::id.eq(folders::bucket_id)))
        .left_join(user_organizations::table.on(user_organizations::organization_id.eq(buckets::organization_id)))
        .filter(user_organizations::user_id.eq(user_id))
        .left_join(organizations::table.on(organizations::id.eq(buckets::organization_id)))
        .select((File::as_select(), Option::<Folder>::as_select(), Option::<Bucket>::as_select(), Option::<Organization>::as_select(), Option::<UserOrganization>::as_select()))
        .get_result::<(File, Option<Folder>, Option<Bucket>, Option<Organization>, Option<UserOrganization>)>(&mut conn)
        .await?;
    if user_org.is_none() || organization.is_none() || bucket.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "No access to this file".to_string()))
    }
    let bucket = bucket.unwrap();
    let organization = organization.unwrap();

    let mut path = "files/".to_owned() + &organization.name + "/" + &bucket.name;
    path.push_str(&folder_path(file.folder_id, &mut conn).await?);
    path.push_str(file.name.as_str());
    let path = Path::new(&path);

    actix_files::NamedFile::open_async(&path).await
        .map_err(|_| ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Cannot open file".to_string()))
}

pub async fn delete_file(file_id: Uuid, user_id: Uuid) -> Result<(), ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    error!("{}", file_id);
    let (file, bucket, organization, user_org) = files::table.find(file_id)
        .left_join(folders::table.on(folders::id.eq(files::folder_id)))
        .left_join(buckets::table.on(buckets::id.eq(folders::bucket_id)))
        .left_join(user_organizations::table.on(user_organizations::organization_id.eq(buckets::organization_id)))
        .filter(user_organizations::user_id.eq(user_id))
        .left_join(organizations::table.on(organizations::id.eq(buckets::organization_id)))
        .select((File::as_select(), Option::<Bucket>::as_select(), Option::<Organization>::as_select(), Option::<UserOrganization>::as_select()))
        .get_result::<(File, Option<Bucket>, Option<Organization>, Option<UserOrganization>)>(&mut conn)
        .await?;
    if user_org.is_none() || organization.is_none() || bucket.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "No access to this file".to_string()))
    }
    let _ = diesel::delete(files::table.find(file_id)).execute(&mut conn).await?;
    let bucket = bucket.unwrap();
    let organization = organization.unwrap();

    let mut path = "files/".to_owned() + &organization.name + "/" + &bucket.name;
    path.push_str(&folder_path(file.folder_id, &mut conn).await?);
    drop(conn);
    path.push_str(file.name.as_str());
    let path = Path::new(&path);
    tokio::fs::remove_file(path).await
        .map_err(|_| ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Cannot remove file".to_string()))

}