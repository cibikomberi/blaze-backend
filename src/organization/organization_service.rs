use actix_web::http::StatusCode;
use diesel::{dsl, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgTextExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    config::db_config, error::ApiResponse, organization::organization_model::{Organization, OrganizationRole, UserOrganization}, schema::{organizations, user_organizations}, user::user_model::User
};
use crate::schema::users;

pub async fn create(name: String, user: &User) -> Result<Organization, ApiResponse> {
    let org = Organization::new(name, user.id);
    let user_org = UserOrganization::new(org.id, user.id,OrganizationRole::OWNER);
    let mut conn = db_config::get_connection().await?;
    conn.transaction::<Organization, ApiResponse, _>(|mut conn| {
            Box::pin(async move {
                let org = diesel::insert_into(crate::schema::organizations::dsl::organizations)
                    .values(org)
                    .get_result::<Organization>(&mut conn)
                    .await?;
                diesel::insert_into(crate::schema::user_organizations::dsl::user_organizations)
                    .values(user_org)
                    .get_result::<UserOrganization>(&mut conn)
                    .await?;
                Ok(org)
            })
        }).await
}

pub async fn list_organizations(user: &User, keyword: Option<String>, limit: i64, cursor: Option<Uuid>) -> Result<Vec<Organization>, ApiResponse> {
    // let query = crate::schema::organizations
    let mut conn = db_config::get_connection().await?;
    let mut query = organizations::table
        .left_join(user_organizations::table)
        .filter(user_organizations::user_id.eq(user.id))
        .into_boxed();

    if let Some(keyword) = keyword {
        query = query.filter(organizations::name.ilike(format!("%{}%", keyword)));
    }
    if let Some(cursor) = cursor {
        query = query.filter(organizations::id.gt(cursor));
    }
    let organizations = query
        .limit(limit)
        .select(Organization::as_select())
        .load::<Organization>(&mut conn)
        .await?;

    Ok(organizations)
}

pub async fn fetch_organization(organization_id: Uuid, user: &User) -> Result<Organization, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (organization, user_organization) = organizations::table.find(organization_id)
        .left_join(user_organizations::table)
        .select((Organization::as_select(), Option::<UserOrganization>::as_select()))
        .get_result::<(Organization, Option<UserOrganization>)>(&mut conn)
        .await?;
    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()));
    }
    Ok(organization)
}

pub async fn list_users_in_organization(organization_id: Uuid, keyword: Option<String>, limit: i64, cursor: Option<Uuid>, user: &User) -> Result<Vec<User>, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (_, user_organization) = validate_access(organization_id ,user.id, &mut conn).await?;
    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()));
    }

    let mut query = organizations::table
        .filter(organizations::id.eq(organization_id)) // instead of find()
        .inner_join(user_organizations::table.on(user_organizations::organization_id.eq(organizations::id)))
        .inner_join(users::table.on(users::id.eq(user_organizations::user_id)))
        .into_boxed();
    if let Some(keyword) = keyword {
        query = query.filter(users::username.ilike(format!("%{}%", keyword))
            .or(users::name.ilike(format!("%{}%", keyword)))
            .or(users::email.ilike(format!("%{}%", keyword))));
    }
    if let Some(cursor) = cursor {
        query = query.filter(users::id.gt(cursor));
    }
        // .filter(users::name.ilike(format!("%{}%", user.name)))
        let users = query.limit(limit)
            .select(User::as_select()) // now valid because join introduces `users`
        .load::<User>(&mut conn).await?;
    Ok(users)
}

pub async fn add_user(user_id: Uuid, organization_id: Uuid, role: OrganizationRole, user: &User) -> Result<User, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (_, user_organization) = validate_access(organization_id ,user.id, &mut conn).await?;
    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()));
    }
    let new_user = users::table.find(user_id)
        // .left_join(user_organizations::table.on(user_organizations::user_id.eq(users::id)))
        // .filter(user_organizations::organization_id.eq(organization_id))
        // .select((User::as_select(), Option::<UserOrganization>::as_select()))
        // .get_result::<(User, Option<UserOrganization>)>(&mut conn).await?;
        .get_result::<User>(&mut conn).await?;
    println!("New user: {:?}", new_user);
    println!("User organization: {:?}", user_organization);
    let _ = dsl::insert_into(user_organizations::table)
        .values(UserOrganization::new(organization_id, new_user.id, role))
        .get_result::<UserOrganization>(&mut conn)
        .await?;
    Ok(new_user)
}

pub async fn validate_access(organization_id: Uuid, user_id: Uuid, conn: &mut AsyncPgConnection) -> Result<(Organization, Option<UserOrganization>), ApiResponse> {
    // let mut conn = db_config::get_connection().await?;
    let org = organizations::table
        .filter(organizations::id.eq(organization_id))
        .left_join(user_organizations::table)
        .filter(user_organizations::user_id.eq(user_id))
        .select((Organization::as_select(), Option::<UserOrganization>::as_select()))
        .first::<(Organization, Option<UserOrganization>)>(conn)
        .await?;

    Ok(org)
}