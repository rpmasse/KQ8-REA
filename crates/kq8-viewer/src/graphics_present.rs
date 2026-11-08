//! A native-mode request needs a frame submitted at its requested size.
//! Camera and Window dimensions alone can match before surface acquisition
//! fails. This bridge observes Bevy's actual acquired texture and its
//! consumption by render_system; it does not claim GPU completion or scanout.
use bevy::{
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        renderer::render_system, sync_world::MainEntity, view::ExtractedWindow,
    },
};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Ticket {
    pub(super) epoch: u64,
    pub(super) window: Entity,
    pub(super) pixels: [u32; 2],
}

#[derive(Resource, Default)]
struct Request {
    epoch: u64,
    pending: Option<Ticket>,
}

/// Copied during extraction, so a render of an older frame cannot acquire
/// the identity of a request issued concurrently by the main world.
#[derive(Resource, Default)]
struct ExtractedRequest(Option<Ticket>);

#[derive(Resource, Clone, Default)]
struct Submitted(Arc<Mutex<Option<Ticket>>>);

impl Submitted {
    fn read(&self) -> Option<Ticket> {
        *self.0.lock().unwrap()
    }

    fn publish(&self, ticket: Ticket) {
        let mut submitted = self.0.lock().unwrap();
        if submitted.is_none_or(|previous| previous.epoch <= ticket.epoch) {
            *submitted = Some(ticket);
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Candidate {
    ticket: Ticket,
    render_window: Entity,
}

impl Candidate {
    fn acquired(
        ticket: Ticket,
        render_window: Entity,
        main_window: Entity,
        texture_pixels: Option<[u32; 2]>,
    ) -> Option<Self> {
        (main_window == ticket.window && texture_pixels == Some(ticket.pixels))
            .then_some(Self { ticket, render_window })
    }

    fn submitted(self, remaining: Option<(Entity, bool)>) -> Option<Ticket> {
        // A disappeared window or an unconsumed texture is not presentation.
        (remaining == Some((self.ticket.window, false))).then_some(self.ticket)
    }
}

#[derive(Resource, Default)]
struct Acquired(Option<Candidate>);

pub(super) fn register(app: &mut App) {
    let submitted = Submitted::default();
    app.init_resource::<Request>().insert_resource(submitted.clone());
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app
            .insert_resource(submitted)
            .init_resource::<ExtractedRequest>()
            .init_resource::<Acquired>()
            .add_systems(ExtractSchedule, extract_request)
            .add_systems(Render, (
                capture_acquired.in_set(RenderSystems::Render).before(render_system),
                confirm_submitted.in_set(RenderSystems::Render).after(render_system),
            ));
    }
}

pub(super) fn issue(world: &mut World, window: Entity, pixels: [u32; 2]) -> Ticket {
    let mut request = world.resource_mut::<Request>();
    request.epoch = request.epoch.checked_add(1).expect("presentation epoch exhausted");
    let ticket = Ticket { epoch: request.epoch, window, pixels };
    request.pending = Some(ticket);
    ticket
}

pub(super) fn complete(world: &mut World, ticket: Ticket) {
    let mut request = world.resource_mut::<Request>();
    if request.pending == Some(ticket) {
        request.pending = None;
    }
}

pub(super) fn observed(world: &World) -> Option<Ticket> {
    world.get_resource::<Submitted>().and_then(Submitted::read)
}

pub(super) fn acknowledged(world: &World, ticket: Ticket) -> bool {
    observed(world) == Some(ticket)
}

fn extract_request(request: Extract<Res<Request>>, mut extracted: ResMut<ExtractedRequest>) {
    extracted.0 = request.pending;
}

fn capture_acquired(
    request: Res<ExtractedRequest>,
    windows: Query<(Entity, MainEntity, &ExtractedWindow)>,
    mut acquired: ResMut<Acquired>,
) {
    acquired.0 = request.0.and_then(|ticket| windows.iter().find_map(|(entity, main, window)| {
        let pixels = window.swap_chain_texture.as_ref().map(|texture| {
            let size = texture.texture.size();
            [size.width, size.height]
        });
        Candidate::acquired(ticket, entity, main, pixels)
    }));
}

fn confirm_submitted(
    mut acquired: ResMut<Acquired>,
    windows: Query<(MainEntity, &ExtractedWindow)>,
    submitted: Res<Submitted>,
) {
    let Some(candidate) = acquired.0.take() else { return };
    let remaining = windows.get(candidate.render_window).ok()
        .map(|(main, window)| (main, window.swap_chain_texture.is_some()));
    if let Some(ticket) = candidate.submitted(remaining) {
        submitted.publish(ticket);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These are supplied texture/presentation observations, not GPU claims.
    #[test]
    fn graphics_present_epoch_window_size_and_restoration_matrix() {
        let mut app = App::new();
        register(&mut app);
        let window = app.world_mut().spawn_empty().id();
        let other = app.world_mut().spawn_empty().id();
        let render_window = app.world_mut().spawn_empty().id();
        let initial = issue(app.world_mut(), window, [640, 480]);
        assert!(!acknowledged(app.world(), initial));
        let bridge = app.world().resource::<Submitted>().clone();
        let candidate = Candidate::acquired(initial, render_window, window, Some([640, 480])).unwrap();
        bridge.publish(candidate.submitted(Some((window, false))).unwrap());
        assert!(acknowledged(app.world(), initial));
        complete(app.world_mut(), initial);

        let high = issue(app.world_mut(), window, [800, 600]);
        assert!(!acknowledged(app.world(), high), "old-size submission must not satisfy a new request");
        for (entity, size) in [(other, Some([800, 600])), (window, Some([640, 480])), (window, None)] {
            assert!(Candidate::acquired(high, render_window, entity, size).is_none());
        }
        let high_candidate = Candidate::acquired(high, render_window, window, Some([800, 600])).unwrap();
        assert!(high_candidate.submitted(Some((window, true))).is_none(), "acquisition without presentation stays pending");
        assert!(high_candidate.submitted(None).is_none(), "window removal is not presentation");
        assert!(high_candidate.submitted(Some((other, false))).is_none());

        // A failed high request restores the same pixels as startup, but
        // requires a fresh submission of its own, not the old startup frame.
        let restoration = issue(app.world_mut(), window, [640, 480]);
        assert!(restoration.epoch > high.epoch && high.epoch > initial.epoch);
        assert!(!acknowledged(app.world(), restoration));
        bridge.publish(high_candidate.submitted(Some((window, false))).unwrap());
        assert!(!acknowledged(app.world(), restoration), "late preceding render stays stale");
        let restored = Candidate::acquired(restoration, render_window, window, Some([640, 480])).unwrap();
        bridge.publish(restored.submitted(Some((window, false))).unwrap());
        assert!(acknowledged(app.world(), restoration));
        bridge.publish(initial);
        assert!(acknowledged(app.world(), restoration), "late old output must not replace newer evidence");
        complete(app.world_mut(), high);
        assert_eq!(app.world().resource::<Request>().pending, Some(restoration));
        complete(app.world_mut(), restoration);
        assert!(app.world().resource::<Request>().pending.is_none());
    }

    #[test]
    fn graphics_present_unacquired_render_schedule_does_not_acknowledge() {
        let mut app = App::new();
        register(&mut app);
        let window = app.world_mut().spawn_empty().id();
        let ticket = issue(app.world_mut(), window, [640, 480]);
        // No acquired texture/window can produce a submission even when the
        // extraction carries matching requested dimensions.
        app.insert_resource(ExtractedRequest(Some(ticket)))
            .init_resource::<Acquired>()
            .add_systems(Update, (capture_acquired, confirm_submitted).chain());
        app.update();
        assert!(!acknowledged(app.world(), ticket));
        assert!(app.world().resource::<Acquired>().0.is_none());
        app.update();
        assert!(!acknowledged(app.world(), ticket));
    }
}
