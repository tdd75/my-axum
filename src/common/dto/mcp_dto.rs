use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SearchUsersParams {
    #[schemars(description = "Filter users by an email substring.")]
    pub email: Option<String>,
    #[schemars(description = "Filter users by a first-name substring.")]
    pub first_name: Option<String>,
    #[schemars(description = "Filter users by a last-name substring.")]
    pub last_name: Option<String>,
    #[schemars(description = "One-based result page. Defaults to 1 when omitted.")]
    pub page: Option<u64>,
    #[schemars(
        description = "Number of users per page. Defaults to the API default when omitted."
    )]
    pub page_size: Option<u64>,
    #[schemars(
        description = "Sort expression such as '+id', '-created_at', or '+email'. Prefix '+' for ascending and '-' for descending."
    )]
    pub order_by: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetUserParams {
    #[schemars(description = "Numeric user id to read.")]
    pub id: i32,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ProfileOutput {
    #[schemars(description = "Numeric id of the authenticated user.")]
    pub id: i32,
    #[schemars(description = "Email address of the authenticated user.")]
    pub email: String,
    #[schemars(description = "Optional first name stored on the profile.")]
    pub first_name: Option<String>,
    #[schemars(description = "Optional last name stored on the profile.")]
    pub last_name: Option<String>,
    #[schemars(description = "Optional phone number stored on the profile.")]
    pub phone: Option<String>,
    #[schemars(description = "Profile creation timestamp serialized by the API.")]
    pub created_at: Option<String>,
    #[schemars(description = "Profile update timestamp serialized by the API.")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UserSummaryOutput {
    #[schemars(description = "Numeric user id.")]
    pub id: i32,
    #[schemars(description = "User email address.")]
    pub email: String,
    #[schemars(description = "Optional first name stored on the user.")]
    pub first_name: Option<String>,
    #[schemars(description = "Optional last name stored on the user.")]
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UserOutput {
    #[schemars(description = "Numeric user id.")]
    pub id: i32,
    #[schemars(description = "User email address.")]
    pub email: String,
    #[schemars(description = "Optional first name stored on the user.")]
    pub first_name: Option<String>,
    #[schemars(description = "Optional last name stored on the user.")]
    pub last_name: Option<String>,
    #[schemars(description = "Optional phone number stored on the user.")]
    pub phone: Option<String>,
    #[schemars(description = "User creation timestamp serialized by the API.")]
    pub created_at: Option<String>,
    #[schemars(description = "User update timestamp serialized by the API.")]
    pub updated_at: Option<String>,
    #[schemars(description = "User that created this record, when available.")]
    pub created_user: Option<UserSummaryOutput>,
    #[schemars(description = "User that last updated this record, when available.")]
    pub updated_user: Option<UserSummaryOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct UserListOutput {
    #[schemars(description = "Users in the current page.")]
    pub items: Vec<UserOutput>,
    #[schemars(description = "Total number of users matching the query.")]
    pub count: usize,
}
