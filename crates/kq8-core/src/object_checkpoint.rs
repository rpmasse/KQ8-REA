//! Identity of an ordinary object made by a script's loadKQ.
//! Model and script bytes are read again from the user's original installation.
use crate::collide::Placement;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ObjectCopy {
    pub file: String,
    pub name: String,
    pub placement: Placement,
}

impl ObjectCopy {
    pub fn valid(&self) -> bool {
        !self.file.is_empty() && self.file.len() <= 256
            && self.file.to_ascii_lowercase().ends_with(".kq")
            && !self.file.contains(['/', '\\', ':']) && !self.file.chars().any(char::is_control)
            && !self.name.is_empty() && self.name.len() <= 1024 && !self.name.chars().any(char::is_control)
            && self.placement.position.iter().chain(self.placement.rotation.iter().flatten()).all(|n| n.is_finite())
    }
}

pub fn valid_copies(copies: &[ObjectCopy]) -> bool {
    let mut names = std::collections::BTreeSet::new();
    copies.len() <= 16384 && copies.iter().all(|copy| copy.valid() && names.insert(copy.name.to_ascii_lowercase()))
}
