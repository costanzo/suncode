use crate::model::SkillInfo;

pub fn render_available_skills<I>(skills: I) -> String
where
    I: IntoIterator<Item = SkillInfo>,
{
    let skills = skills.into_iter().collect::<Vec<_>>();
    if skills.is_empty() {
        return String::new();
    }
    let mut output = String::from("Skills provide specialized instructions and workflows for specific tasks. Use the skill tool to load a matching skill.\n\n<available_skills>\n");
    for skill in skills {
        output.push_str(&format!("  <skill>\n    <name>{}</name>\n    <description>{}</description>\n    <location>{}</location>\n  </skill>\n", escape_xml(&skill.name), escape_xml(&skill.description), escape_xml(&skill.location.to_string_lossy())));
    }
    output.push_str("</available_skills>");
    output
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SkillSource;
    use std::path::PathBuf;

    #[test]
    fn renders_escaped_available_skill_metadata() {
        let info = SkillInfo {
            name: "a-b".into(),
            description: "a < b".into(),
            location: PathBuf::from("/tmp/a"),
            base_directory: PathBuf::from("/tmp"),
            source: SkillSource::Explicit,
            explicit_only: false,
        };
        let output = render_available_skills([info]);
        assert!(output.contains("a &lt; b"));
    }
}
