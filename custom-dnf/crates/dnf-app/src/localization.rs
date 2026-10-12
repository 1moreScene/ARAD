//! Chinese SKL description overlays; original PVF combat columns remain authoritative.
use dnf_data::skill::SkillDefinition;
pub fn apply(skill: &mut SkillDefinition) -> Result<(), String> {
    let script = match skill.id {
        5 => include_str!("../../../data/skill/swordman/hardattack.skl"),
        46 => include_str!("../../../data/skill/swordman/upperslash.skl"),
        31 => include_str!("../../../data/skill/swordman/grabblastblood.skl"),
        _ => return Ok(()),
    };
    let translated = SkillDefinition::parse(skill.id, &skill.path, script)?;
    for tag in [
        "name",
        "name2",
        "explain",
        "level property",
        "command key explain",
    ] {
        if let Some(text) = translated.fields.get(tag) {
            skill.fields.insert(tag.into(), text.clone());
        }
    }
    Ok(())
}
