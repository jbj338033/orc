use super::types::{Skill, SkillContext};

pub fn parse_skill_md(name: &str, content: &str) -> Option<Skill> {
    let (frontmatter, body) = extract_frontmatter(content)?;

    let description = extract_field(&frontmatter, "description").unwrap_or_default();
    let disable_model = extract_field(&frontmatter, "disable-model-invocation")
        .map(|v| v == "true")
        .unwrap_or(false);
    let context = match extract_field(&frontmatter, "context").as_deref() {
        Some("fork") => SkillContext::Fork,
        _ => SkillContext::Inline,
    };
    let allowed_tools = extract_field(&frontmatter, "allowed-tools").map(|v| {
        v.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    });
    let triggers = extract_field(&frontmatter, "triggers")
        .map(|v| {
            v.split(',')
                .map(|s| s.trim().trim_matches('"').to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    Some(Skill {
        name: name.to_string(),
        description,
        prompt: body.trim().to_string(),
        allowed_tools,
        disable_model_invocation: disable_model,
        context,
        triggers,
    })
}

fn extract_frontmatter(content: &str) -> Option<(String, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Some((String::new(), content.to_string()));
    }

    let after_first = &trimmed[3..];
    let end = after_first.find("---")?;
    let frontmatter = after_first[..end].to_string();
    let body = after_first[end + 3..].to_string();
    Some((frontmatter, body))
}

fn extract_field(frontmatter: &str, key: &str) -> Option<String> {
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix(':') {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}
