use super::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct State {
    partner: String,
    partner_at: [f32; 3],
    info: Option<String>,
    lines: Speak,
    keep_weapon: bool,
    case: Option<Case>,
    stage: Stage,
    walk_from: [f32; 3],
    approach_updates: u32,
    cuts_stopped: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedConversation {
    active: Option<State>,
    leave: Option<String>,
    hands_off: bool,
}

impl SavedConversation {
    pub fn beginning_bare_hand_out(&self) -> bool {
        self.active.as_ref().is_some_and(|c| c.stage == Stage::Out && c.approach_updates == 0)
    }

    pub fn valid(&self) -> bool {
        let name = |n: &str| !n.is_empty() && n.len() <= 1024 && !n.chars().any(char::is_control);
        self.leave.as_deref().is_none_or(name) && self.active.as_ref().is_none_or(|c| {
            name(&c.partner) && c.info.as_deref().is_none_or(name)
                && c.lines.issuer.eq_ignore_ascii_case(&c.partner) && c.lines.first.seq > 0 && c.lines.last >= c.lines.first.seq
                && c.partner_at.iter().chain(&c.walk_from).all(|n| n.is_finite())
                && c.approach_updates < u32::MAX / 2
        })
    }
}

impl WorldState {
    pub fn saved_conversation(&self) -> SavedConversation {
        SavedConversation { active: self.conversation.as_ref().map(|c| State { partner: c.partner.clone(), partner_at: c.partner_at,
            info: c.info.as_ref().map(|i| i.name.clone()), lines: c.lines.clone(), keep_weapon: c.keep_weapon,
            case: c.case, stage: c.stage, walk_from: c.walk_from, approach_updates: c.approach_updates, cuts_stopped: c.cuts_stopped }),
            leave: self.connor_leave.clone(), hands_off: self.connor_hands_off }
    }

    pub fn restore_conversation(&mut self, saved: &SavedConversation) -> Result<(), String> {
        if !saved.valid() { return Err("invalid conversation checkpoint".into()); }
        let active = saved.active.as_ref().map(|c| -> Result<Conversation, String> {
            if !self.known.contains(&c.partner.to_ascii_lowercase()) { return Err(format!("saved conversation partner '{}' is absent", c.partner)); }
            let info = c.info.as_ref().map(|name| self.conv_infos.iter().find(|i| i.name.eq_ignore_ascii_case(name)).cloned()
                .ok_or_else(|| format!("saved conversation info '{name}' is absent"))).transpose()?;
            Ok(Conversation { partner: c.partner.clone(), partner_at: c.partner_at, info, lines: c.lines.clone(), keep_weapon: c.keep_weapon,
                case: c.case, stage: c.stage, walk_from: c.walk_from, approach_updates: c.approach_updates,
                cuts_stopped: c.cuts_stopped, at: "disk conversation checkpoint".into() })
        }).transpose()?;
        self.conversation = active;
        self.connor_leave = saved.leave.clone();
        self.connor_hands_off = saved.hands_off;
        self.conversation_note(format!("conversation checkpoint restored: {:?}; no entry or partner-start replayed", self.conversation.as_ref().map(|c| (&c.partner, c.stage, c.approach_updates))));
        Ok(())
    }
}
