//! Original Graphics device policy (graphics-modes.md rules 1–10).
//!
//! Inputs describe an explicitly supplied legacy device inventory. A modern
//! WGPU adapter does not establish any of these families or capabilities.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Family {
    Software,
    Direct3D,
    Glide,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlideBoard {
    pub board_type: u32,
    pub framebuffer_mb: u32,
}

impl GlideBoard {
    pub fn supports_800(self) -> bool {
        matches!(self.board_type, 0 | 1) && self.framebuffer_mb >= 4
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Device {
    pub name: String,
    pub family: Family,
    pub fullscreen: bool,
    /// The original capability query uses the first reported Glide board.
    pub first_glide_board: Option<GlideBoard>,
}

impl Device {
    pub fn software(fullscreen: bool) -> Self {
        Self { name: "Software".into(), family: Family::Software, fullscreen, first_glide_board: None }
    }

    pub fn direct3d(driver: &str, fullscreen: bool) -> Self {
        Self { name: format!("D3D-{driver}"), family: Family::Direct3D, fullscreen, first_glide_board: None }
    }

    pub fn glide(fullscreen: bool, first_board: Option<GlideBoard>) -> Self {
        Self { name: "Glide".into(), family: Family::Glide, fullscreen, first_glide_board: first_board }
    }

    pub fn supports_800(&self) -> bool {
        self.family == Family::Glide && self.first_glide_board.is_some_and(GlideBoard::supports_800)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Inventory {
    devices: Vec<Device>,
}

/// Requests leave the active device and preferences unchanged until the
/// presentation owner accepts a live change or accepts the restart decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Selection {
    Live(Device),
    ConfirmSoftwareRestart,
}

/// Q207: refresh updates eligible targets but retains a family's previous
/// target when that row becomes unavailable. Availability is not this store.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Targets {
    software: Option<Device>,
    direct3d: Option<Device>,
    glide: Option<Device>,
}

impl Targets {
    pub fn refresh(&mut self, inventory: &Inventory, high_resolution: bool) {
        for family in [Family::Software,Family::Direct3D,Family::Glide] {
            if let Some(device) = inventory.choice(family,high_resolution) {
                let target = match family {
                    Family::Software => &mut self.software,
                    Family::Direct3D => &mut self.direct3d,
                    Family::Glide => &mut self.glide,
                };
                *target = Some(device.clone());
            }
        }
    }

    /// Stored-focus activation is allowed even while the row is disabled.
    /// Err identifies a never-resolved target: original unchecked access has
    /// no defined result, so callers report a safe failure without a fallback.
    pub fn activate(&self, active: &Device, family: Family) -> Result<Option<Selection>,Family> {
        if family == Family::Software && self.direct3d.as_ref().is_some_and(|d| d.family == active.family) {
            return Ok(Some(Selection::ConfirmSoftwareRestart));
        }
        let target = match family {
            Family::Software => self.software.as_ref(),
            Family::Direct3D => self.direct3d.as_ref(),
            Family::Glide => self.glide.as_ref(),
        }.ok_or(family)?;
        if target.family == active.family { Ok(None) }
        else { Ok(Some(Selection::Live(target.clone()))) }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupGap {
    EmptyInventory,
    UnknownOrUnavailableDevice,
    Glide800Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupRequest {
    pub name: String,
    pub high_resolution: bool,
    /// No fallback is invented for these untraced cases. A request is not
    /// evidence that a renderer was created successfully.
    pub unresolved: Option<StartupGap>,
}

impl Inventory {
    pub fn new(devices: Vec<Device>) -> Self { Self { devices } }

    /// Fixed rows retain their positions even when this returns None.
    pub fn choice(&self, family: Family, high_resolution: bool) -> Option<&Device> {
        if family == Family::Software && high_resolution { return None; }
        if family == Family::Direct3D && self.devices.iter().any(|d| d.family == Family::Glide) {
            return None;
        }
        // Software and Glide have canonical names; Direct3D may enumerate
        // multiple drivers and the last eligible one is the original choice.
        self.devices.iter().rev().find(|d| d.family == family && d.fullscreen
            && (family != Family::Direct3D || d.name != "D3D-dd3dfx"))
    }

    /// A fresh eligible-row selection. A persistent Graphics screen uses
    /// Targets::activate instead, after retaining targets across refreshes.
    pub fn select(&self, active: &Device, family: Family, high_resolution: bool) -> Option<Selection> {
        let target = self.choice(family, high_resolution)?;
        // Q207 follow-up: the selected row and no-op compare renderer
        // families, even if enumeration retained a different D3D driver.
        if target.family == active.family { return None; }
        if family == Family::Software && active.family == Family::Direct3D
            && self.choice(Family::Direct3D, high_resolution).is_some()
        {
            Some(Selection::ConfirmSoftwareRestart)
        } else {
            Some(Selection::Live(target.clone()))
        }
    }

    /// Apply only the established startup name/800 rules. This is separate
    /// from row filtering: e.g. a present Glide can suppress a D3D row without
    /// proving that the previously saved D3D name has an original fallback.
    pub fn startup_request(&self, saved_name: &str, high_resolution: bool) -> StartupRequest {
        if self.devices.is_empty() {
            return StartupRequest { name: saved_name.into(), high_resolution: saved_name == "Glide" && high_resolution,
                unresolved: Some(StartupGap::EmptyInventory) };
        }
        if saved_name == "Glide" {
            if let Some(device) = self.choice(Family::Glide, false) {
                return StartupRequest { name: saved_name.into(), high_resolution,
                    unresolved: (high_resolution && !device.supports_800()).then_some(StartupGap::Glide800Unavailable) };
            }
            return StartupRequest { name: "Software".into(), high_resolution: false, unresolved: None };
        }
        let available = self.devices.iter().any(|d| d.name == saved_name && d.fullscreen);
        StartupRequest { name: saved_name.into(), high_resolution: false,
            unresolved: (!available).then_some(StartupGap::UnknownOrUnavailableDevice) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphics_device_policy_grouped_inventory_capability_selection_and_startup() {
        let software = Device::software(true);
        let first = Device::direct3d("first", true);
        let last = Device::direct3d("display", true);
        let excluded = Device::direct3d("dd3dfx", true);
        let absent = Device::direct3d("unavailable", false);
        let inventory = Inventory::new(vec![software.clone(), first.clone(), last.clone(), excluded, absent]);
        assert_eq!(inventory.choice(Family::Direct3D, false), Some(&last));
        assert_eq!(inventory.choice(Family::Software, false), Some(&software));
        assert_eq!(inventory.choice(Family::Software, true), None);
        assert_eq!(inventory.choice(Family::Glide, false), None);
        assert!(!software.supports_800());
        assert!(!last.supports_800());
        assert_eq!(inventory.select(&software, Family::Direct3D, false), Some(Selection::Live(last.clone())));
        assert_eq!(inventory.select(&last, Family::Direct3D, false), None);
        assert_eq!(inventory.select(&last, Family::Software, false), Some(Selection::ConfirmSoftwareRestart));
        assert_eq!(inventory.select(&last, Family::Software, true), None);
        assert_eq!(inventory.select(&first, Family::Direct3D, false), None);

        for (board_type, framebuffer_mb, high) in [(0,3,false),(0,4,true),(1,4,true),(1,8,true),(2,8,false)] {
            let glide = Device::glide(true, Some(GlideBoard { board_type, framebuffer_mb }));
            assert_eq!(glide.supports_800(), high);
            let devices = Inventory::new(vec![software.clone(), last.clone(), glide.clone()]);
            assert_eq!(devices.choice(Family::Direct3D, false), None, "Glide presence suppresses D3D");
            assert_eq!(devices.select(&last, Family::Software, false), Some(Selection::Live(software.clone())),
                "restart confirmation needs the corresponding D3D menu entry");
            assert_eq!(devices.select(&glide, Family::Software, false), Some(Selection::Live(software.clone())));
            let request = devices.startup_request("Glide", true);
            assert_eq!((request.name.as_str(), request.high_resolution), ("Glide", true));
            assert_eq!(request.unresolved, (!high).then_some(StartupGap::Glide800Unavailable));
        }
        assert!(!Device::glide(true, None).supports_800());
        let unavailable_glide = Inventory::new(vec![software.clone(), last.clone(), Device::glide(false, None)]);
        assert_eq!(unavailable_glide.choice(Family::Direct3D, false), None, "presence is not fullscreen eligibility");
        assert_eq!(unavailable_glide.choice(Family::Glide, false), None);
        assert_eq!(unavailable_glide.startup_request("Glide", true), StartupRequest {
            name: "Software".into(), high_resolution: false, unresolved: None });
        assert_eq!(inventory.startup_request("Glide", true), StartupRequest {
            name: "Software".into(), high_resolution: false, unresolved: None });
        for name in ["Software", "D3D-display"] {
            assert_eq!(inventory.startup_request(name, true), StartupRequest {
                name: name.into(), high_resolution: false, unresolved: None });
        }
        assert_eq!(inventory.startup_request("D3D-missing", true), StartupRequest {
            name: "D3D-missing".into(), high_resolution: false, unresolved: Some(StartupGap::UnknownOrUnavailableDevice) });
        assert_eq!(Inventory::default().startup_request("Glide", true), StartupRequest {
            name: "Glide".into(), high_resolution: true, unresolved: Some(StartupGap::EmptyInventory) });
        assert_eq!(Inventory::default().startup_request("D3D-display", true), StartupRequest {
            name: "D3D-display".into(), high_resolution: false, unresolved: Some(StartupGap::EmptyInventory) });
    }
}
