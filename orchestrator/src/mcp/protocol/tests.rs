use super::*;

#[test]
fn test_serialize_request() {
    let req = McpRequest::new(1, "tools/list", None);
    let json = serde_json::to_string(&req).unwrap();

    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"id\":1"));
    assert!(json.contains("\"method\":\"tools/list\""));
}

#[test]
fn test_deserialize_request() {
    let json = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    let req: McpRequest = serde_json::from_str(json).unwrap();

    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.id, 1);
    assert_eq!(req.method, "tools/list");
    assert!(req.params.is_none());
}

#[test]
fn test_serialize_response_success() {
    let result = serde_json::json!({"tools": []});
    let resp = McpResponse::ok(1, result.clone());
    let json = serde_json::to_string(&resp).unwrap();

    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"id\":1"));
    assert!(json.contains("\"result\""));
    assert!(!json.contains("\"error\""));
}

#[test]
fn test_serialize_response_error() {
    let err = McpError::method_not_found("unknown_method");
    let resp = McpResponse::err(1, err);
    let json = serde_json::to_string(&resp).unwrap();

    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"id\":1"));
    assert!(json.contains("\"error\""));
    assert!(!json.contains("\"result\""));
}

#[test]
fn test_response_is_success() {
    let ok_resp = McpResponse::ok(1, serde_json::json!({}));
    let err_resp = McpResponse::err(1, McpError::internal_error("failed"));

    assert!(ok_resp.is_success());
    assert!(!err_resp.is_success());
}

#[test]
fn test_response_into_result() {
    let result = serde_json::json!({"status": "ok"});
    let ok_resp = McpResponse::ok(1, result.clone());

    assert_eq!(ok_resp.into_result().unwrap(), result);

    let err = McpError::invalid_params("bad params");
    let err_resp = McpResponse::err(1, err.clone());

    assert_eq!(err_resp.into_result().unwrap_err(), err);
}

#[test]
fn test_error_codes() {
    let parse_err = McpError::parse_error("invalid json");
    assert_eq!(parse_err.code, -32700);

    let invalid_req = McpError::invalid_request("bad request");
    assert_eq!(invalid_req.code, -32600);

    let method_nf = McpError::method_not_found("test");
    assert_eq!(method_nf.code, -32601);

    let invalid_params = McpError::invalid_params("bad params");
    assert_eq!(invalid_params.code, -32602);

    let internal = McpError::internal_error("server error");
    assert_eq!(internal.code, -32603);
}

#[test]
fn test_mcp_method_conversion() {
    assert_eq!(McpMethod::Initialize.as_str(), "initialize");
    assert_eq!(McpMethod::ToolsList.as_str(), "tools/list");
    assert_eq!(McpMethod::ToolsCall.as_str(), "tools/call");

    // String to McpMethod
    let method: McpMethod = "tools/list".into();
    assert_eq!(method, McpMethod::ToolsList);

    // Custom method
    let custom: McpMethod = "custom/method".into();
    assert!(matches!(custom, McpMethod::Custom(_)));

    // Test all method variants for coverage
    assert_eq!(McpMethod::ResourcesList.as_str(), "resources/list");
    assert_eq!(McpMethod::ResourcesRead.as_str(), "resources/read");
    assert_eq!(McpMethod::PromptsList.as_str(), "prompts/list");
    assert_eq!(McpMethod::PromptsGet.as_str(), "prompts/get");

    // Test Custom variant with as_str()
    let custom_method = McpMethod::Custom("my/custom".to_string());
    assert_eq!(custom_method.as_str(), "my/custom");
}

#[test]
fn test_tool_serialization() {
    let tool = Tool {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: serde_json::json!({"type": "object"}),
    };

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"name\":\"test_tool\""));
    assert!(json.contains("\"description\":\"A test tool\""));
}

#[test]
fn test_round_trip_request() {
    let original = McpRequest::new(
        42,
        "tools/call",
        Some(serde_json::json!({"name": "test", "args": {}})),
    );

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: McpRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn test_request_with_params() {
    let params = serde_json::json!({"query": "test"});
    let req = McpRequest::new(1, "resources/read", Some(params));

    assert!(req.params.is_some());
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"params\""));
}

#[test]
fn test_request_notification() {
    let req = McpRequest::notification(1, "tools/list");
    assert_eq!(req.id, 1);
    assert_eq!(req.method, "tools/list");
    assert!(req.params.is_none());
}

#[test]
fn test_error_with_data() {
    let data = serde_json::json!({"details": "Additional error info"});
    let err = McpError::with_data(-32000, "Server error", data.clone());

    assert_eq!(err.code, -32000);
    assert_eq!(err.message, "Server error");
    assert_eq!(err.data, Some(data));
}

#[test]
fn test_error_server_error() {
    let err = McpError::server_error("Connection failed");
    assert_eq!(err.code, -32000);
    assert!(err.message.contains("Connection failed"));
}

#[test]
fn test_response_into_result_invalid() {
    // Edge case: response with both result and error (invalid)
    let invalid_resp = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: Some(serde_json::json!({"status": "ok"})),
        error: Some(McpError::internal_error("Error")),
    };

    let result = invalid_resp.into_result();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, -32603);
    assert!(err.message.contains("Invalid response"));
}

#[test]
fn test_error_new() {
    let err = McpError::new(-32001, "Custom error");
    assert_eq!(err.code, -32001);
    assert_eq!(err.message, "Custom error");
    assert!(err.data.is_none());
}

#[test]
fn test_error_invalid_params() {
    let err = McpError::invalid_params("Missing required field");
    assert_eq!(err.code, -32602);
    assert!(err.message.contains("Missing required field"));
}

#[test]
fn test_error_parse_error() {
    let err = McpError::parse_error("Unexpected token");
    assert_eq!(err.code, -32700);
    assert!(err.message.contains("Unexpected token"));
}

#[test]
fn test_error_invalid_request() {
    let err = McpError::invalid_request("Missing jsonrpc field");
    assert_eq!(err.code, -32600);
    assert!(err.message.contains("Missing jsonrpc field"));
}
