//! Original reference material bundled into the executable at compile time.
//!
//! Resource URIs select exact registry entries. They are never interpreted as
//! filesystem paths or network locations, so reads cannot access user files.

use rmcp::{
    ErrorData,
    model::{
        ListResourcesResult, ReadResourceRequestParams, ReadResourceResult, Resource,
        ResourceContents,
    },
};

/// Approved core guidance, shared by initialize metadata and resource reads.
pub const CORE_INSTRUCTIONS: &str = include_str!("../../assets/core-instructions.md");

struct BundledResource {
    uri: &'static str,
    name: &'static str,
    description: &'static str,
    text: &'static str,
}

const RESOURCES: &[BundledResource] = &[
    BundledResource {
        uri: "diffyn://guidance/core-instructions",
        name: "Core instructions",
        description: "Preference-first server guidance, privacy boundaries, and tool limitations.",
        text: CORE_INSTRUCTIONS,
    },
    BundledResource {
        uri: "diffyn://guidance/style-guide",
        name: "Communication style guide",
        description: "Adult-respectful language, structure, numerical accuracy, and client presentation guidance.",
        text: include_str!("../../assets/resources/style-guide.md"),
    },
    BundledResource {
        uri: "diffyn://guidance/profile-matrix",
        name: "Combined preference matrix",
        description: "Optional supports and conflicts across self-described neurotypes; no diagnostic classification.",
        text: include_str!("../../assets/resources/profile-matrix.md"),
    },
    BundledResource {
        uri: "diffyn://guidance/evidence-and-limits",
        name: "Evidence and accessibility limits",
        description: "Source provenance, evidence strength, implementation boundaries, and evaluation requirements.",
        text: include_str!("../../assets/resources/evidence-and-limits.md"),
    },
];

/// Discover every resource with its actual UTF-8 byte size and MIME type.
pub fn list_resources() -> ListResourcesResult {
    ListResourcesResult::with_all_items(
        RESOURCES
            .iter()
            .map(|resource| {
                Resource::new(resource.uri, resource.name)
                    .with_description(resource.description)
                    .with_mime_type("text/markdown")
                    .with_size(resource.text.len() as u64)
            })
            .collect(),
    )
}

/// Read only an exact allowlisted URI; no decoding, redirects, or path joining.
pub fn read_resource(params: ReadResourceRequestParams) -> Result<ReadResourceResult, ErrorData> {
    if params.input_responses.is_some() || params.request_state.is_some() {
        return Err(ErrorData::invalid_params(
            "Bundled resources do not accept retry state or input responses.",
            None,
        ));
    }
    if params.uri.len() > 32 * 1024 {
        return Err(ErrorData::invalid_params(
            "Resource URI exceeds the 32768-byte limit.",
            None,
        ));
    }
    let resource = RESOURCES
        .iter()
        .find(|resource| resource.uri == params.uri)
        .ok_or_else(|| {
            // Do not echo the URI: a rejected file/network URI can contain a
            // private path, credential, identifier, or supplied source text.
            ErrorData::resource_not_found(
                "Unknown resource URI. Use resources/list for available resources.",
                None,
            )
        })?;
    Ok(ReadResourceResult::new(vec![
        ResourceContents::text(resource.text, resource.uri).with_mime_type("text/markdown"),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_listed_resources_resolve_with_matching_metadata() -> Result<(), ErrorData> {
        let registry = list_resources();
        assert_eq!(registry.resources.len(), 4);
        for resource in registry.resources {
            assert_eq!(resource.mime_type.as_deref(), Some("text/markdown"));
            let result = read_resource(ReadResourceRequestParams::new(&resource.uri))?;
            assert_eq!(result.contents.len(), 1);
            assert!(matches!(
                result.contents.first(),
                Some(ResourceContents::TextResourceContents { uri, text, mime_type, .. })
                    if uri == &resource.uri
                        && mime_type.as_deref() == Some("text/markdown")
                        && Some(text.len() as u64) == resource.size
                        && !text.trim().is_empty()
            ));
        }
        Ok(())
    }

    #[test]
    fn file_network_traversal_and_variant_uris_are_never_resolved() {
        for uri in [
            "file:///etc/passwd",
            "https://example.com/private",
            "diffyn://guidance/../../etc/passwd",
            "diffyn://guidance/%2e%2e/core-instructions",
            "diffyn://guidance/core-instructions?secret=PRIVATE",
            "diffyn://guidance/core-instructions#part",
            "DIFFYN://guidance/core-instructions",
            "diffyn://guidance/core-instructions/",
            "",
        ] {
            let result = read_resource(ReadResourceRequestParams::new(uri));
            assert!(matches!(
                result,
                Err(error) if error.message == "Unknown resource URI. Use resources/list for available resources."
                    && error.data.is_none()
            ));
        }
    }

    #[test]
    fn oversized_uri_and_retry_state_are_rejected() {
        assert!(read_resource(ReadResourceRequestParams::new("x".repeat(32769))).is_err());
        let params = ReadResourceRequestParams::new(RESOURCES[0].uri).with_request_state("private");
        assert!(matches!(read_resource(params), Err(error) if !error.message.contains("private")));
    }

    #[test]
    fn core_resource_and_server_instruction_constant_are_identical() {
        assert_eq!(RESOURCES[0].text, CORE_INSTRUCTIONS);
        assert!(CORE_INSTRUCTIONS.contains("higher-priority"));
        assert!(CORE_INSTRUCTIONS.contains("Do not log or persist"));
    }
}
