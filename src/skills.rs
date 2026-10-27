//! Versioned workflow skills served from embedded, allowlisted files.
//!
//! The Skills extension describes discovery and retrieval. The host decides
//! whether and how to load a skill; a resource read never grants permissions.

use std::collections::BTreeMap;

use rmcp::{
    ErrorData,
    model::{
        CacheScope, CustomRequest, CustomResult, ReadResourceRequestParams, ReadResourceResult,
        ResourceContents,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const EXTENSION_ID: &str = "io.modelcontextprotocol/skills";
const CACHE_TTL_MS: u64 = 300_000;

struct BundledSkill {
    name: &'static str,
    uri: &'static str,
    text: &'static str,
}

macro_rules! bundled_skill {
    ($name:literal) => {
        BundledSkill {
            name: $name,
            uri: concat!("skill://", $name, "/SKILL.md"),
            text: include_str!(concat!("../skills/", $name, "/SKILL.md")),
        }
    };
}

const SKILLS: &[BundledSkill] = &[
    bundled_skill!("diffyn-understand-document"),
    bundled_skill!("diffyn-task-support"),
    bundled_skill!("diffyn-repair-communication"),
    bundled_skill!("diffyn-write-in-my-voice"),
    bundled_skill!("diffyn-make-decision"),
    bundled_skill!("diffyn-prepare-conversation"),
];

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListParams {
    cursor: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GetParams {
    uri: String,
}

fn invalid_params() -> ErrorData {
    ErrorData::invalid_params(
        "Invalid skill request. Use skills/list and the exact published skill URI.",
        None,
    )
}

fn invalid_bundle() -> ErrorData {
    ErrorData::internal_error("The bundled skill metadata could not be read.", None)
}

/// Bundled front matter deliberately uses only single-line plain string fields.
/// These are build-owned documents, not caller-supplied YAML. Tests enforce this
/// subset so discovery preserves every field without a runtime YAML parser.
fn frontmatter(skill: &BundledSkill) -> Result<BTreeMap<&str, &str>, ErrorData> {
    let mut lines = skill.text.lines();
    if lines.next() != Some("---") {
        return Err(invalid_bundle());
    }
    let mut fields = BTreeMap::new();
    let mut closed = false;
    for line in lines {
        if line == "---" {
            closed = true;
            break;
        }
        let (key, value) = line.split_once(": ").ok_or_else(invalid_bundle)?;
        if !matches!(key, "name" | "description" | "license")
            || value.is_empty()
            || value != value.trim()
            || value.starts_with(['\'', '"', '|', '>', '[', '{'])
            || value.contains(": ")
            || value.contains(" #")
            || fields.insert(key, value).is_some()
        {
            return Err(invalid_bundle());
        }
    }
    if !closed
        || fields.get("name") != Some(&skill.name)
        || fields
            .get("description")
            .is_none_or(|value| value.len() > 1024)
        || fields.get("license") != Some(&"MIT")
    {
        return Err(invalid_bundle());
    }
    Ok(fields)
}

fn entry(skill: &BundledSkill) -> Result<Value, ErrorData> {
    Ok(json!({
        "uri": skill.uri,
        "frontmatter": frontmatter(skill)?,
        "resources": [{
            "uri": skill.uri,
            "digest": format!("sha256:{}", Sha256::digest(skill.text.as_bytes())
                .iter().map(|byte| format!("{byte:02x}")).collect::<String>()),
            "size": skill.text.len(),
        }],
    }))
}

pub fn handle_request(request: CustomRequest) -> Result<CustomResult, ErrorData> {
    match request.method.as_str() {
        "skills/list" => {
            let params: ListParams =
                serde_json::from_value(request.params.unwrap_or_else(|| json!({})))
                    .map_err(|_| invalid_params())?;
            if params.cursor.is_some() {
                return Err(invalid_params());
            }
            let skills: Result<Vec<_>, _> = SKILLS.iter().map(entry).collect();
            Ok(CustomResult(json!({
                "resultType": "complete", "skills": skills?,
                "ttlMs": CACHE_TTL_MS, "cacheScope": "public",
            })))
        }
        "skills/get" => {
            let params: GetParams =
                serde_json::from_value(request.params.ok_or_else(invalid_params)?)
                    .map_err(|_| invalid_params())?;
            let skill = SKILLS
                .iter()
                .find(|skill| skill.uri == params.uri)
                .ok_or_else(invalid_params)?;
            Ok(CustomResult(json!({
                "resultType": "complete", "skill": entry(skill)?,
                "ttlMs": CACHE_TTL_MS, "cacheScope": "public",
            })))
        }
        _ => Err(ErrorData::new(
            rmcp::model::ErrorCode::METHOD_NOT_FOUND,
            "The requested method is not available.",
            None,
        )),
    }
}

/// Exact matching prevents traversal, decoding, remote reads and directory access.
pub fn read_resource(
    params: &ReadResourceRequestParams,
) -> Option<Result<ReadResourceResult, ErrorData>> {
    let skill = SKILLS.iter().find(|skill| skill.uri == params.uri)?;
    Some(
        if params.input_responses.is_some() || params.request_state.is_some() {
            Err(invalid_params())
        } else {
            Ok(ReadResourceResult::new(vec![
                ResourceContents::text(skill.text, skill.uri).with_mime_type("text/markdown"),
            ])
            .with_ttl_ms(CACHE_TTL_MS)
            .with_cache_scope(CacheScope::Public))
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_frontmatter_preserves_metadata_and_manifest_bytes() -> Result<(), ErrorData> {
        let skill = BundledSkill {
            name: "diffyn-example",
            uri: "skill://diffyn-example/SKILL.md",
            text: "---\r\nname: diffyn-example\r\ndescription: An example workflow.\r\nlicense: MIT\r\n---\r\nRead the source.\r\n",
        };
        let metadata = entry(&skill)?;
        assert_eq!(
            metadata["frontmatter"]["description"],
            "An example workflow."
        );
        assert_eq!(metadata["resources"][0]["size"], skill.text.len());
        let digest: String = Sha256::digest(skill.text.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(
            metadata["resources"][0]["digest"],
            format!("sha256:{digest}")
        );
        Ok(())
    }

    #[test]
    fn skill_metadata_and_manifests_describe_the_served_files() -> Result<(), ErrorData> {
        let listed = handle_request(CustomRequest::new("skills/list", None))?.0;
        assert_eq!(listed["skills"].as_array().map(Vec::len), Some(6));
        for skill in SKILLS {
            let metadata = entry(skill)?;
            let fetched = handle_request(CustomRequest::new(
                "skills/get",
                Some(json!({"uri":skill.uri})),
            ))?
            .0;
            assert_eq!(fetched["skill"], metadata);
            assert_eq!(metadata["frontmatter"]["name"], skill.name);
            assert_eq!(metadata["resources"][0]["size"], skill.text.len());
            assert!(skill.text.ends_with('\n'));
            assert!(matches!(
                read_resource(&ReadResourceRequestParams::new(skill.uri)),
                Some(Ok(_))
            ));
        }
        Ok(())
    }

    #[test]
    fn unknown_paths_fields_and_cursors_do_not_leak_input() {
        for (method, params) in [
            ("skills/list", json!({"cursor":"PRIVATE"})),
            ("skills/list", json!({"PRIVATE":true})),
            ("skills/get", json!({"uri":"file:///PRIVATE"})),
            (
                "skills/get",
                json!({"uri":"skill://diffyn-task-support/../PRIVATE"}),
            ),
            ("skills/get", json!({"uri":SKILLS[0].uri,"PRIVATE":true})),
            (
                "skills/get",
                json!({"uri":SKILLS[0].uri,"requestState":"PRIVATE"}),
            ),
        ] {
            let result = handle_request(CustomRequest::new(method, Some(params)));
            assert!(
                matches!(result, Err(error) if !error.message.contains("PRIVATE") && error.data.is_none())
            );
        }
        for uri in [
            "file:///PRIVATE",
            "skill://diffyn-task-support/SKILL.md/",
            "skill://diffyn-task-support/%53KILL.md",
        ] {
            assert!(read_resource(&ReadResourceRequestParams::new(uri)).is_none());
        }
    }
}
