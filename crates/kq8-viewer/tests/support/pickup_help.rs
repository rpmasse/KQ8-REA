/// Match the notice against the real pickup and its later native panel.
/// A busy Connor, another popup or a recent delivery can postpone opening.
pub fn check(items: &str, viewer: &str, kind: &str, case: u8, delay_updates: u64, heading: &str) {
    let request = format!("{kind} added: inventory help case {case}, first eligible update ");
    let mut requests = items.lines().filter_map(|s| s.split_once(&request));
    let (prefix, due) = requests.next().expect("pickup scheduled its original help");
    assert!(requests.next().is_none(), "one pickup notice for {kind}");
    let received: u64 = prefix
        .trim_start_matches("update ")
        .trim_end_matches(": ")
        .parse()
        .unwrap();
    let due: u64 = due.parse().unwrap();
    assert_eq!(due, received + delay_updates);

    let opened = format!("starting help: 500.MSG case {case} opened on update ");
    let mut pages = viewer.lines().filter_map(|l| l.strip_prefix(&opened));
    let (tick, contents) = pages
        .next()
        .expect("original pickup tutorial opened")
        .split_once(':')
        .unwrap();
    let tick: u64 = tick.parse().unwrap();
    assert!(tick >= due, "case {case} opened early");
    assert!(
        contents.contains(heading) && contents.contains("checkbox true"),
        "{contents}"
    );
    assert!(pages.next().is_none(), "duplicate tutorial {case}");
    assert!(viewer.contains(&format!(
        "acknowledged case {case} on update {tick}, help enabled true"
    )));
}
