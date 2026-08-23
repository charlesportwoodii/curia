use curia::{Filter, Level, Threshold};

#[test]
fn a_bare_directive_sets_the_global_threshold() {
    let filter = Filter::from_directives("warn");

    assert!(filter.admits("anything", Level::Error));
    assert!(filter.admits("anything", Level::Warn));
    assert!(!filter.admits("anything", Level::Info));
}

#[test]
fn off_admits_nothing_for_a_matching_target() {
    let filter = Filter::from_directives("info,hyper=off");

    assert!(filter.admits("bvc_server_lib::http", Level::Info));
    assert!(!filter.admits("hyper::client", Level::Error));
}

#[test]
fn a_longer_prefix_wins_over_a_shorter_one() {
    let filter = Filter::from_directives("info,rocket=off,rocket::server::x=trace");

    assert!(!filter.admits("rocket::server", Level::Error));
    assert!(filter.admits("rocket::server::x::y", Level::Trace));
}

#[test]
fn an_unparseable_directive_is_skipped_and_the_rest_still_apply() {
    let filter = Filter::from_directives("info,hyper=verbose,rustls=off");

    assert!(filter.admits("hyper::client", Level::Info));
    assert!(!filter.admits("rustls::conn", Level::Error));
}

#[test]
fn the_default_filter_admits_everything() {
    let filter = Filter::default();

    assert!(filter.admits("anything", Level::Trace));
}

#[test]
fn a_directive_string_with_no_global_defaults_to_info() {
    let filter = Filter::from_directives("hyper=off");

    assert!(filter.admits("bvc", Level::Info));
    assert!(!filter.admits("bvc", Level::Debug));
}

#[test]
fn a_threshold_of_off_admits_no_level_at_all() {
    assert!(!Threshold::Off.admits(Level::Error));
    assert!(Threshold::At(Level::Error).admits(Level::Error));
}
