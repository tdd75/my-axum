use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use rust_i18n::t;

use crate::common::dto::mcp_dto::{
    GetUserParams, ProfileOutput, SearchUsersParams, UserListOutput, UserOutput,
};
use crate::common::util::mcp_util::api_client::{HttpApiClient, internal_mcp_error};

#[derive(Clone)]
pub struct McpServer {
    api_client: HttpApiClient,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("my-axum", env!("CARGO_PKG_VERSION")).with_title("My Axum"),
            )
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_instructions(t!("mcp.instructions", locale = "en").to_string())
    }
}

#[tool_router]
impl McpServer {
    pub fn new(api_base_url: String) -> Self {
        Self {
            api_client: HttpApiClient::new(api_base_url),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Return the authenticated user's profile.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<ProfileOutput>()
    )]
    async fn get_current_user_profile(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let data = self
            .api_client
            .get(&ctx, "/api/v1/user/profile/", None)
            .await?;
        Ok(CallToolResult::structured(data))
    }

    #[tool(
        description = "Search users by email, name, pagination, and ordering. Requires Admin role.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<UserListOutput>()
    )]
    async fn search_users(
        &self,
        Parameters(params): Parameters<SearchUsersParams>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let query = serde_urlencoded::to_string(params).map_err(internal_mcp_error)?;
        let data = self
            .api_client
            .get(&ctx, "/api/v1/user/", Some(&query))
            .await?;
        Ok(CallToolResult::structured(data))
    }

    #[tool(
        description = "Return one user by id. Requires Admin role.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<UserOutput>()
    )]
    async fn get_user(
        &self,
        Parameters(GetUserParams { id }): Parameters<GetUserParams>,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let data = self
            .api_client
            .get(&ctx, &format!("/api/v1/user/{id}/"), None)
            .await?;
        Ok(CallToolResult::structured(data))
    }
}
