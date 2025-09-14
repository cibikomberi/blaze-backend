use crate::bucket::bucket_model::Bucket;
use crate::config::db_config;
use crate::error::ApiResponse;
use crate::folder::folder_dto::Entry;
use crate::folder::folder_model::{Folder, FolderId};
use crate::organization::organization_model::{Organization, OrganizationRole, UserOrganization};
use crate::organization::organization_service;
use crate::schema::folders;
use crate::schema::user_organizations;
use crate::schema::{buckets, organizations};
use crate::user::user_model::User;
use actix_web::http::StatusCode;
use diesel::sql_types::{BigInt, Nullable, Text};
use diesel::sql_types::{Uuid as SqlUuid};
use diesel::{sql_query, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lazy_static::lazy_static;
use std::option::Option;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

lazy_static! {
    pub static ref EDITABLE_ROLES: [OrganizationRole; 3] = [OrganizationRole::OWNER, OrganizationRole::ADMIN, OrganizationRole::EDITOR];
}
pub async fn create(name: String, bucket_id: Uuid, parent_id: Uuid, user: &User) -> Result<Folder, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let buc = buckets::table.find(bucket_id)
        .first::<Bucket>(&mut conn)
        .await?;
    let (organization, user_organization) = organization_service::validate_access(buc.organization_id, user.id, &mut conn).await
        .map_err(|_| ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()))?;
    match user_organization {
        Some(user_organization) if EDITABLE_ROLES.contains(&user_organization.role) => (),
        _ => return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()))
    }
    let parent_folder = folders::dsl::folders.find(parent_id)
        .first::<Folder>(&mut conn)
        .await?;
    if parent_folder.bucket_id != bucket_id {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "Folder is not in this bucket".to_string()));
    }

    let folder = diesel::insert_into(folders::dsl::folders)
        .values(Folder::new(name, bucket_id, Some(parent_id), user.id))
        .get_result::<Folder>(&mut conn)
        .await?;

    let mut path = "files/".to_owned() + &organization.name + "/" + &buc.name;
    path.push_str(&folder_path(folder.id, &mut conn).await?);
    let path = Path::new(&path);
    println!("{:?}", path);
    let _ = fs::create_dir_all(path).await;
    // folder_with_parents(folder.id).await;
    Ok(folder)
}

pub async fn folder_path(folder_id: Uuid, conn: &mut AsyncPgConnection) -> Result<String, ApiResponse> {
    let query = r#"
    WITH RECURSIVE folder_chain AS (
        SELECT * FROM folders WHERE id = $1
        UNION ALL
        SELECT f.* FROM folders f
        INNER JOIN folder_chain fc ON fc.parent_id = f.id
    )
    SELECT * FROM folder_chain;
"#;

    let results: Vec<Folder> = diesel::sql_query(query)
        .bind::<diesel::sql_types::Uuid, _>(folder_id)
        .load(conn)
        .await
        .ok().unwrap();

    let mut path = "/".to_string();
    results.into_iter().rev().for_each(|folder| path.push_str(&(folder.name + "/")));
    Ok(path)
}

pub async fn create_folder_from_path(path: &str, bucket_id: Uuid, created_by: Uuid, conn: &mut AsyncPgConnection) -> Result<Uuid, ApiResponse> {
    let query = r#"SELECT create_folders_for_path($1, $2, $3) as id;"#;
    let folder: FolderId = diesel::sql_query(query)
        .bind::<diesel::sql_types::Uuid, _>(bucket_id)
        .bind::<Text, _>(path)
        .bind::<diesel::sql_types::Uuid, _>(created_by)
        .get_result(conn)
        .await?;
    Ok(folder.id)
}

pub async fn get_folder_from_path(path: &str, bucket: Bucket, conn: &mut AsyncPgConnection) -> Result<Option<Uuid>, ApiResponse> {
    let query = r#"SELECT folder_exists_for_path($1, $2) as id;"#;
    let folder: Option<FolderId> = sql_query(query)
        .bind::<diesel::sql_types::Uuid, _>(bucket.id)
        .bind::<Text, _>(path)
        .get_result::<Option<FolderId>>(conn)
        .await?;

    match folder {
        Some(folder) => Ok(Some(folder.id)),
        None => Ok(None),
    }
}

pub async fn find(bucket_id: Uuid, folder_id: Option<Uuid>,keyword: Option<String>, limit: i64, cursor: Option<Uuid>, cursor_kind: String, user: &User) -> Result<(Folder, Vec<Entry>), ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (_, user_org) = buckets::dsl::buckets.find(bucket_id)
        .left_join(user_organizations::table.on(user_organizations::organization_id.eq(buckets::organization_id)))
        .filter(user_organizations::user_id.eq(user.id))
        .select((Bucket::as_select(), Option::<UserOrganization>::as_select()))
        .first::<(Bucket, Option<UserOrganization>)>(&mut conn)
        .await?;

    if user_org.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()))
    }
    let folder;
    if let Some(folder_id) = folder_id {
        folder = folders::dsl::folders.find(folder_id)
            .filter(folders::bucket_id.eq(bucket_id))
            .get_result::<Folder>(&mut conn)
            .await?;
    } else {
        folder = folders::dsl::folders.filter(folders::bucket_id.eq(bucket_id))
            .filter(folders::parent_id.is_null())
            .first::<Folder>(&mut conn)
            .await?;
    }
    let keyword_pattern = format!("%{}%", keyword.unwrap_or_default());

    let cursor_id = cursor.unwrap_or(Uuid::nil());
    let folder_cursor_condition = if cursor_kind == "folder" { format!("AND public.folders.id < '{cursor_id}'") } else if cursor_kind == "file" { "AND FALSE".to_string() } else { "".to_string() };
    let file_cursor_condition = if cursor_kind == "file" { format!("AND public.files.id < '{cursor_id}'") } else { "".to_string() };
let query = format!(r#"
        SELECT public.folders.id as id, public.folders.name as name, 'folder' as kind, public.folders.created_at as created_at, created_by, users.name as user_name, users.username as user_username, users.email as user_email
FROM folders
LEFT JOIN users ON users.id = folders.created_by
    WHERE parent_id = $2
  AND public.folders.name ILIKE $3
  {folder_cursor_condition}

UNION ALL

SELECT public.files.id as id, public.files.name as name, 'file' as kind, public.files.created_at as created_at, created_by, users.name as user_name, users.username as user_username, users.email as user_email
FROM files
LEFT JOIN users ON users.id = files.created_by

WHERE folder_id = $2
  AND public.files.name ILIKE $3
  {file_cursor_condition}

ORDER BY kind DESC, id DESC
LIMIT $4
    "#);
    let results = sql_query(query)
        .bind::<SqlUuid, _>(bucket_id)
        .bind::<Nullable<SqlUuid>, _>(Some(folder.id))
        .bind::<Text, _>(keyword_pattern)
        .bind::<BigInt, _>(limit)
        .load::<Entry>(&mut conn)
        .await?;

    Ok((folder, results))
}

pub async fn delete_folder(folder_id: Uuid,user_id: Uuid) -> Result<(), ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (folder, bucket, organization, user_org) = folders::table.find(folder_id)
        .left_join(buckets::table.on(buckets::id.eq(folders::bucket_id)))
        .left_join(user_organizations::table.on(user_organizations::organization_id.eq(buckets::organization_id)))
        .filter(user_organizations::user_id.eq(user_id))
        .left_join(organizations::table.on(organizations::id.eq(buckets::organization_id)))
        .select((Folder::as_select(), Option::<Bucket>::as_select(), Option::<Organization>::as_select(), Option::<UserOrganization>::as_select()))
        .get_result::<(Folder, Option<Bucket>, Option<Organization>, Option<UserOrganization>)>(&mut conn)
        .await?;
    if user_org.is_none() || organization.is_none() || bucket.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "No access to this file".to_string()))
    }
    let bucket = bucket.unwrap();
    let organization = organization.unwrap();

    let mut path = "files/".to_owned() + &organization.name + "/" + &bucket.name;
    path.push_str(&folder_path(folder.id, &mut conn).await?);
    let _ = diesel::delete(folders::table.find(folder_id)).execute(&mut conn).await?;
    drop(conn);
    let path = Path::new(&path);
    let _ = fs::remove_dir_all(path).await.unwrap();
    Ok(())
}