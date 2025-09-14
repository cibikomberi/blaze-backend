use crate::bucket::bucket_model::{Bucket, BucketVisibility};
use crate::config::db_config;
use crate::error::ApiResponse;
use crate::file::file_dto::FileQueryDto;
use crate::file::file_model::File;
use crate::folder::folder_model::Folder;
use crate::folder::folder_service::folder_path;
use crate::organization::organization_model::{Organization, OrganizationRole, OrganizationSecret, UserOrganization};
use crate::organization::organization_service;
use crate::schema::files;
use crate::schema::organization_secrets;
use crate::schema::{buckets, user_organizations};
use crate::schema::{folders, organizations};
use crate::user::user_model::User;
use crate::folder::folder_service;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use actix_web::Responder;
use diesel::{ExpressionMethods, PgTextExpressionMethods, SelectableHelper};
use diesel::{JoinOnDsl, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use hmac::{Mac};
use lazy_static::lazy_static;
use sha2::Sha256;
use std::path::Path;
use std::time::{SystemTime};
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
    let (_folder, user_organization) = folders::dsl::folders.find(folder_id)
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
    let (file, _folder, bucket, organization, user_org) = files::table.find(file_id)
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

pub async fn serve_file(organization_name: String, bucket_name: String, file_path: String, query: FileQueryDto) -> Result<actix_files::NamedFile, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (organization, bucket) = find_organization_and_bucket(&organization_name, &bucket_name, &mut conn).await?;
    let path = organization_name.to_string() + "/" + &bucket_name + "/" + &file_path;
    if bucket.visibility == BucketVisibility::PRIVATE {
        let _ = verify_signature(&path, query, organization, false, &mut conn).await?;
    }
    actix_files::NamedFile::open_async("files/".to_string() + &path).await
        .map_err(|_| ApiResponse::new(StatusCode::NOT_FOUND, "File not found".to_string()))
}

pub async fn save_file(body: Bytes, organization_name: String, bucket_name: String, file_path: String, query: FileQueryDto) -> Result<(), ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (organization, bucket) = find_organization_and_bucket(&organization_name, &bucket_name, &mut conn).await?;

    let path = organization_name.to_string() + "/" + &bucket_name + "/" + &file_path;
    let org_sec = verify_signature(&path, query, organization, true, &mut conn).await?;

    let path = Path::new(&file_path);

    let parent = path.parent().map(|p| p.to_str().unwrap()).unwrap_or("");
    let folder_id = folder_service::create_folder_from_path(parent, bucket.id, org_sec.created_by, &mut conn).await?;

    let file = path.file_name().map(|f| f.to_str().unwrap()).unwrap_or("");

    let _file = diesel::insert_into(files::table)
        .values(File::new(file.to_string(), folder_id, org_sec.created_by))
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await?;

    let actual_file_path = format!("files/{}/{}/{}", organization_name, bucket_name, file_path);
    let path = Path::new(&actual_file_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    let _ = fs::remove_file(&actual_file_path).await;
    let _ = fs::File::create(&actual_file_path).await
        .map_err(|_| ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Cannot create file".to_string()))?
        .write_all(&body).await
        .map_err(|_| ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Cannot write to file".to_string()))?;
    Ok(())
}

pub async fn remove_file(organization_name: String, bucket_name: String, file_path: String, query: FileQueryDto) -> Result<(), ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (organization, bucket) = find_organization_and_bucket(&organization_name, &bucket_name, &mut conn).await?;
    let path = organization_name.to_string() + "/" + &bucket_name + "/" + &file_path;
    let _org_sec = verify_signature(&path, query, organization, true, &mut conn).await?;

    let path = Path::new(&file_path);

    let parent = path.parent().map(|p| p.to_str().unwrap()).unwrap_or("");
    let folder: Option<Uuid> = folder_service::get_folder_from_path(parent, bucket, &mut conn).await?;

    let folder = match folder {
        Some(folder_id) => folder_id,
        None => return Err(ApiResponse::new(StatusCode::FORBIDDEN, "Folder not found".to_string()))
    };

    let file = path.file_name().map(|f| f.to_str().unwrap()).unwrap_or("");
    let file = diesel::delete(files::table)
        .filter(files::folder_id.eq(folder))
        .filter(files::name.eq(file))
        .execute(&mut conn)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Cannot delete file".to_string()))?;
    if file < 1 {
        return Err(ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, "Cannot delete file".to_string()));
    }
    let actual_file_path = format!("files/{}/{}/{}", organization_name, bucket_name, file_path);
    let _ = tokio::fs::remove_file(actual_file_path).await;
    Ok(())
}

async fn find_organization_and_bucket(organization_name: &str, bucket_name: &str, conn: &mut AsyncPgConnection) -> Result<(Organization, Bucket), ApiResponse> {
    let (org, bucket) = organizations::table
        .left_join(buckets::table.on(buckets::organization_id.eq(organizations::id)))
        .filter(organizations::name.eq(&organization_name))
        .filter(buckets::name.eq(&bucket_name))
        .select((Option::<Organization>::as_select(), Option::<Bucket>::as_select()))
        .first::<(Option<Organization>, Option<Bucket>)>(conn)
        .await?;
    if org.is_none() || bucket.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "No access to this file".to_string()))
    }
    Ok((org.unwrap(), bucket.unwrap()))
}


async fn verify_signature(path: &str, query: FileQueryDto, org: Organization, is_upload: bool, conn: &mut AsyncPgConnection) -> Result<OrganizationSecret, ApiResponse> {
    let FileQueryDto { expiry, secret_id, signature} = query;
    let org_secret = organization_secrets::table
        .filter(organization_secrets::id.eq(&secret_id))
        .filter(organization_secrets::organization_id.eq(org.id))
        .select(OrganizationSecret::as_select())
        .first::<OrganizationSecret>(conn)
        .await?;

    let s = format!(
        "{path}?{}secret_id={secret_id}{}",
        if is_upload { "upload=true&" } else { "" },
        expiry.map(|e| format!("&expiry={e}")).unwrap_or_default(),
    );


    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(org_secret.secret.as_bytes()).unwrap();
    mac.update(s.as_bytes());
    let fin = mac.finalize().into_bytes();

    let hex_str = hex::encode(fin);
    if  hex_str != signature {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "Secret not matched".to_string()))
    }

    if let Some(expiry) = expiry {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if expiry < now {
            return Err(ApiResponse::new(StatusCode::FORBIDDEN, "URL expired".to_string()));
        }
    }
    Ok(org_secret)
}