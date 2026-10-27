//! MCP registration and privacy boundary for the local Diffyn capabilities.

pub mod transport;

use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::ToolCallContext, wrapper::Parameters},
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, CustomRequest,
        CustomResult, ErrorCode, GetPromptRequestParams, GetPromptResponse, Implementation,
        ListPromptsResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
        ReadResourceRequestParams, ReadResourceResponse, ServerCapabilities, ServerConfig,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::Serialize;

use crate::{prompts, resources, skills, tools};

/// A stateless handler: no user profile, message history, or storage is retained.
#[derive(Clone)]
pub struct DiffynServer {
    tool_router: ToolRouter<Self>,
}

impl Default for DiffynServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl DiffynServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    /// Format local text conservatively using explicit preferences. The output
    /// preserves the source for review; it is not a medical or semantic assessment.
    #[tool(
        description = "Reformat text locally for cognitive accessibility using declared preferences. Input is limited to 32768 UTF-8 bytes. Preserves the original and reports changes and limits; does not diagnose or guarantee comprehension.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::ReformatOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn reformat_for_cognitive_load(
        &self,
        Parameters(request): Parameters<tools::ReformatRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::reformat_for_cognitive_load(request))
    }

    /// Extract explicit steps without claiming a general-purpose task planner.
    #[tool(
        description = "Build a local executive-function checklist from explicit task text, limited to 32768 UTF-8 bytes. Separates extracted actions from suggested planning steps and missing information. Cannot infer a reliable chronology or safety plan from arbitrary prose.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::ChecklistOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn generate_executive_function_checklist(
        &self,
        Parameters(request): Parameters<tools::ChecklistRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::generate_executive_function_checklist(request))
    }

    /// Offer finite-pattern clarity findings while retaining the original text.
    #[tool(
        description = "Check up to 32768 UTF-8 bytes locally against a finite set of potentially ambiguous or idiomatic phrases. Returns reviewable findings and original text. Does not reliably detect sarcasm, prove clarity, or silently rewrite meaning.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::ClarityOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn verify_clarity_and_literalism(
        &self,
        Parameters(request): Parameters<tools::ClarityRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::verify_clarity_and_literalism(request))
    }

    #[tool(
        description = "Resolve explicit communication preferences across general, task and current-request scopes. Reports effective settings, their sources, conflicts and host-only guidance. No diagnosis inference, learning or storage.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::ResolvePreferencesOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn resolve_communication_preferences(
        &self,
        Parameters(request): Parameters<tools::ResolvePreferencesRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::resolve_communication_preferences(request))
    }

    #[tool(
        description = "Compare original and candidate text for changes to exact numerical details, URLs, code and user-marked protected passages. Returns source evidence and coverage limits; never certifies equivalent meaning or completeness.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::CompareDetailsOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn compare_protected_details(
        &self,
        Parameters(request): Parameters<tools::CompareDetailsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::compare_protected_details(request))
    }

    #[tool(
        description = "Check a supplied structured action plan for invalid identifiers, source references and dependencies, including dependency cycles. Preserves the distinction between source actions and proposals. Does not establish procedural safety or completeness.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::ValidatePlanOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn validate_action_plan(
        &self,
        Parameters(request): Parameters<tools::ValidatePlanRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::validate_action_plan(request))
    }

    #[tool(
        description = "Build a portable resume card from an explicitly supplied goal and task state. Retains confirmed status, blockers and the requested return point. Does not infer completion, remember previous calls or save the card.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::ResumeCardOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn build_resume_card(
        &self,
        Parameters(request): Parameters<tools::ResumeCardRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::build_resume_card(request))
    }

    #[tool(
        description = "Calculate bounded decimal values, percentage changes and supported unit conversions locally. Retains the supplied values, calculation and any rounding. Does not infer units, dates, financial advice or risk meaning.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::NumericOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn explain_numeric_values(
        &self,
        Parameters(request): Parameters<tools::NumericRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::explain_numeric_values(request))
    }

    #[tool(
        description = "Check text against explicit measurable presentation requirements such as paragraph limits, headings, terminology and plain text. Identifies which checks ran and their limits; no cognitive-load score or accessibility certification.",
        output_schema = rmcp::handler::server::tool::schema_for_output::<tools::ResponseContractOutput>(),
        annotations(read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    fn check_response_contract(
        &self,
        Parameters(request): Parameters<tools::ResponseContractRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        structured_result(tools::check_response_contract(request))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for DiffynServer {
    fn get_info(&self) -> ServerConfig {
        // Advertise only implemented capabilities; no sampling, logging,
        // subscriptions, network access, or persistent profiles are enabled.
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .enable_extensions_with(std::collections::BTreeMap::from([(
                    skills::EXTENSION_ID.to_owned(),
                    serde_json::Map::new(),
                )]))
                .build(),
        )
        .with_server_info(Implementation::new("diffyn", env!("CARGO_PKG_VERSION")))
        .with_instructions(resources::CORE_INSTRUCTIONS)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        if request.request_state.is_some() || request.input_responses.is_some() {
            return Err(ErrorData::invalid_params(
                "Diffyn tools accept a single request and do not support continuation state.",
                None,
            ));
        }

        // SDK deserialization errors may contain submitted enum values or
        // unknown field names. They can arrive either as ErrorData or an isError
        // tool result; sanitize both without echoing any request material.
        let context = ToolCallContext::new(self, request, context);
        match self.tool_router.call(context).await {
            Ok(CallToolResponse::Complete(result)) if result.is_error == Some(true) => {
                Ok(CallToolResult::error(vec![ContentBlock::text(INVALID_ARGUMENTS)]).into())
            }
            Ok(result) => Ok(result),
            Err(error) => Err(sanitize_tool_error(error)),
        }
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        validate_cursor(request)?;
        Ok(ListToolsResult::with_all_items(self.tool_router.list_all()))
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        validate_cursor(request)?;
        Ok(prompts::list_prompts())
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, ErrorData> {
        prompts::get_prompt(request).map(Into::into)
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        validate_cursor(request)?;
        Ok(resources::list_resources())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        if let Some(result) = skills::read_resource(&request) {
            return result.map(Into::into);
        }
        resources::read_resource(request).map(Into::into)
    }

    async fn on_custom_request(
        &self,
        request: CustomRequest,
        _context: RequestContext<RoleServer>,
    ) -> Result<CustomResult, ErrorData> {
        if matches!(request.method.as_str(), "skills/list" | "skills/get") {
            return skills::handle_request(request);
        }
        // The SDK default repeats the unknown method string, which can itself
        // contain private input. Unsupported methods need only a fixed error.
        Err(ErrorData::new(
            ErrorCode::METHOD_NOT_FOUND,
            "The requested method is not available.",
            None,
        ))
    }
}

const INVALID_ARGUMENTS: &str =
    "Tool arguments are invalid. Check the published input schema and request limits.";

/// Return machine-readable content plus the SDK's JSON text compatibility block.
/// Serialize only completed outputs; failures must never expose source content.
fn structured_result<T: Serialize>(result: Result<T, String>) -> Result<CallToolResult, ErrorData> {
    let output = result.map_err(|_| ErrorData::invalid_params(INVALID_ARGUMENTS, None))?;
    let value = serde_json::to_value(output)
        .map_err(|_| ErrorData::internal_error("Diffyn could not encode the tool result.", None))?;
    Ok(CallToolResult::structured(value))
}

fn sanitize_tool_error(error: ErrorData) -> ErrorData {
    let message = match error.code {
        ErrorCode::INVALID_PARAMS => INVALID_ARGUMENTS,
        ErrorCode::METHOD_NOT_FOUND => "The requested tool is not available.",
        _ => "Diffyn could not complete the tool request.",
    };
    ErrorData::new(error.code, message, None)
}

/// All registries fit in one response, so no continuation cursor is valid.
fn validate_cursor(request: Option<PaginatedRequestParams>) -> Result<(), ErrorData> {
    if request.is_some_and(|params| params.cursor.is_some()) {
        Err(ErrorData::invalid_params(
            "This registry has one page and does not accept a cursor.",
            None,
        ))
    } else {
        Ok(())
    }
}
