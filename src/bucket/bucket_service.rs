use actix_web::http::StatusCode;
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgTextExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use tokio::fs;
use uuid::Uuid;

use crate::folder::folder_model::Folder;
use crate::organization::organization_model::{Organization, UserOrganization};
use crate::schema::{folders, organizations, user_organizations};
use crate::{bucket::bucket_model::Bucket, config::db_config, error::ApiResponse, organization::organization_service, schema::buckets, user::user_model::User};
use crate::bucket::bucket_model::{BucketChangeset, BucketVisibility};
use crate::folder::folder_service::EDITABLE_ROLES;

pub async fn create(name: String, organization_id: Uuid, user: &User) -> Result<Bucket, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (organization, _) = organization_service::validate_access(organization_id, user.id, &mut conn).await
        .map_err(|_| ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()))?;

    let bucket = conn.transaction::<Bucket, ApiResponse, _>(|mut conn| {
        Box::pin(async move {
            let bucket = diesel::insert_into(buckets::table)
                .values(Bucket::new(name, organization_id, user.id))
                .get_result::<Bucket>(&mut conn)
                .await?;
            let _root_folder = diesel::insert_into(folders::table)
                .values(Folder::new("".to_string(), bucket.id, None, user.id))
                .execute(&mut conn)
                .await?;
        Ok(bucket)
        })
    }).await?;

    let _ = tokio::fs::create_dir_all(&("files/".to_string() + &organization.name + "/" + &bucket.name)).await;

    Ok(bucket)
}

pub async fn list(organization_id: Uuid, keyword: Option<String>, limit: i64, cursor: Option<Uuid>, user: &User) -> Result<Vec<Bucket>, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (_, user_organization)  = organization_service::validate_access(organization_id, user.id, &mut conn).await?;
    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "User doesn't have access to this organization".to_string()));
    }
    let mut query = buckets::table
        .filter(buckets::organization_id.eq(organization_id))
        .into_boxed();
    if let Some(keyword) = keyword {
        query = query.filter(buckets::name.ilike(format!("%{}%", keyword)));
    }
    if let Some(cursor) = cursor {
        query = query.filter(buckets::id.gt(cursor));
    }
    let buckets = query.limit(limit)
        .load::<Bucket>(&mut conn)
        .await?;

    Ok(buckets)
}

pub async fn update_bucket(bucket_id: Uuid, name: Option<String>, visibility: Option<BucketVisibility>, user: &User) -> Result<Bucket, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (bucket, user_organization) = buckets::table.find(bucket_id)
        .left_join(user_organizations::table.on(buckets::organization_id.eq(user_organizations::organization_id).and(user_organizations::user_id.eq(user.id))))
        .select((Bucket::as_select(), Option::<UserOrganization>::as_select()))
        .first::<(Bucket, Option<UserOrganization>)>(&mut conn)
        .await?;
    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "User doesn't have access to this organization".to_string()));
    }
    let user_organization = user_organization.unwrap();
    if !EDITABLE_ROLES.contains(&user_organization.role) {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have permission to update a bucket".to_string()));
    }
    let changeset = BucketChangeset { name, visibility };
    let bucket = diesel::update(buckets::table)
        .set(changeset)
        .filter(buckets::id.eq(bucket.id))
        .get_result::<Bucket>(&mut conn)
        .await?;

    Ok(bucket)

}

pub async fn delete_bucket(bucket_id: Uuid, user: &User) -> Result<Bucket, ApiResponse> {
    let mut conn = db_config::get_connection().await.unwrap();
    let (user_organization, organization) = buckets::table.find(bucket_id)
        .left_join(organizations::table.on(organizations::id.eq(buckets::organization_id)))
        .left_join(user_organizations::table.on(user_organizations::organization_id.eq(buckets::organization_id)))
        .filter(user_organizations::user_id.eq(user.id))
        .select((Option::<UserOrganization>::as_select(), Option::<Organization>::as_select()))
        .get_result::<(Option::<UserOrganization>, Option<Organization>)>(&mut conn)
        .await?;

    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "User doesn't have access to this organization".to_string()));
    }
    let bucket = diesel::delete(buckets::table.find(bucket_id)).get_result::<Bucket>(&mut conn).await?;
    drop(conn);
    let path = "files/".to_string() + &organization.unwrap().name + "/" + &bucket.name;
    let _ = fs::remove_dir_all(path).await;
    Ok(bucket)
}