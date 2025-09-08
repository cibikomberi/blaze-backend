use actix_web::http::StatusCode;
use chrono::NaiveDateTime;
use diesel::{alias, dsl, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgTextExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    config::db_config, error::ApiResponse, organization::organization_model::{Organization, OrganizationRole, UserOrganization}, schema::{organizations, user_organizations}, user::user_model::User
};
use crate::folder::folder_service::EDITABLE_ROLES;
use crate::organization::organization_dto::{OrganizationUserRoleDto, UserDto};
use crate::schema::users;

pub async fn create(name: String, user: &User) -> Result<Organization, ApiResponse> {
    let org = Organization::new(name, user.id);
    let user_org = UserOrganization::new(org.id, user.id,OrganizationRole::OWNER, None);
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

pub async fn fetch_organization(organization_id: Uuid, _user: &User) -> Result<Organization, ApiResponse> {
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

pub async fn list_users_in_organization(organization_id: Uuid, keyword: Option<String>, limit: i64, cursor: Option<Uuid>, user: &User) -> Result<Vec<OrganizationUserRoleDto>, ApiResponse> {
    let users_added_by = alias!(users as added_by_users);

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
        let users: Vec<OrganizationUserRoleDto> = query.limit(limit)
            .left_join(
                users_added_by.on(
                    user_organizations::added_by.eq(users_added_by.field(users::id).nullable())
                )
            )
            .select((
                users::id,
                users::name,
                users::email,
                users::username,
                user_organizations::role,
                user_organizations::added_at,

                    users_added_by.field(users::id).nullable(),
                    users_added_by.field(users::name).nullable(),
                    users_added_by.field(users::email).nullable(),
                    users_added_by.field(users::username).nullable(),

            ))
            .load::<(Uuid, String, String, String, OrganizationRole, NaiveDateTime, Option<Uuid>, Option<String>, Option<String>, Option<String>)>(&mut conn).await?
    .into_iter()
        .map(|(user_id, name, email, username, role, added_at, added_id, added_name, added_email, added_username)| {
            OrganizationUserRoleDto {
                user_id,
                name,
                email,
                username,
                role,
                added_at,
                added_by: added_id.map(|id| UserDto {
                    user_id: id,
                    name: added_name.unwrap_or_default(),
                    email: added_email.unwrap_or_default(),
                    username: added_username.unwrap_or_default(),
                }),
            }
        })
        .collect();
    Ok(users)
}

pub async fn add_user(user_id: Uuid, organization_id: Uuid, role: OrganizationRole, user: &User) -> Result<User, ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (_, user_organization) = validate_access(organization_id ,user.id, &mut conn).await?;
    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()));
    }
    let (new_user, user_org) = users::table.find(user_id)
        .left_join(user_organizations::table.on(user_organizations::user_id.eq(users::id).and(user_organizations::organization_id.eq(organization_id))))
        .select((User::as_select(), Option::<UserOrganization>::as_select()))
        .get_result::<(User, Option<UserOrganization>)>(&mut conn).await?;
        // .get_result::<User>(&mut conn).await?;
        error!("here1");

    if user_org.is_some() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "User is already part of the organization".to_string()));
    }

    let _ = dsl::insert_into(user_organizations::table)
        .values(UserOrganization::new(organization_id, user_id, role, Some(user.id)))
        .get_result::<UserOrganization>(&mut conn)
        .await?;

    Ok(new_user)
}

pub async fn update_user(user_id: Uuid, organization_id: Uuid, role: OrganizationRole, user: &User) -> Result<User, ApiResponse> {
    if role == OrganizationRole::OWNER {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "Cannot change role to owner".to_string()));
    }
    let mut conn = db_config::get_connection().await?;
    let (_, user_organization) = validate_access(organization_id ,user.id, &mut conn).await?;
    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()));
    }
    if let Some(user_organization) = user_organization {
        if !EDITABLE_ROLES.contains(&user_organization.role) {
            return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()));
        }
    }
    let (user, user_org) = users::table.find(user_id)
        .left_join(user_organizations::table.on(user_organizations::user_id.eq(users::id).and(user_organizations::organization_id.eq(organization_id))))
        .select((User::as_select(), Option::<UserOrganization>::as_select()))
        .get_result::<(User, Option<UserOrganization>)>(&mut conn)
        .await?;
    if user_org.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "User is not a part of this organization".to_string()));
    }
    let user_org = user_org.unwrap();
    if user_org.role == OrganizationRole::OWNER {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "Cannot change owner".to_string()));
    }
    let _ = diesel::update(user_organizations::table)
        .filter(user_organizations::organization_id.eq(organization_id))
        .filter(user_organizations::user_id.eq(user.id))
        .set(user_organizations::role.eq(role))
        .execute(&mut conn)
        .await?;

    Ok(user)
}

pub async fn delete_user(user_id: Uuid, organization_id: Uuid, user: &User) -> Result<(), ApiResponse> {
    let mut conn = db_config::get_connection().await?;
    let (_, user_organization) = validate_access(organization_id ,user.id, &mut conn).await?;
    if user_organization.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()));
    }
    if let Some(user_organization) = user_organization {
        if !EDITABLE_ROLES.contains(&user_organization.role) {
            return Err(ApiResponse::new(StatusCode::FORBIDDEN, "You do not have access to this organization".to_string()));
        }
    }
    let (_, user_org) = users::table.find(user_id)
        .left_join(user_organizations::table.on(user_organizations::user_id.eq(users::id).and(user_organizations::organization_id.eq(organization_id))))
        .select((User::as_select(), Option::<UserOrganization>::as_select()))
        .get_result::<(User, Option<UserOrganization>)>(&mut conn)
        .await?;
    if user_org.is_none() {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "User is not a part of this organization".to_string()));
    }
    let user_org = user_org.unwrap();
    if user_org.role == OrganizationRole::OWNER {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, "Cannot remove owner".to_string()));
    }
    diesel::delete(user_organizations::table.filter(user_organizations::organization_id.eq(organization_id))
        .filter(user_organizations::user_id.eq(user_id)))
        .filter(user_organizations::organization_id.eq(organization_id))
        .execute(&mut conn)
        .await?;
    Ok(())
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