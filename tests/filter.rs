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
fn the_widest_threshold_is_a_verbose_target_rather_than_a_quieter_global() {
    let filter = Filter::from_directives("info,bvc::route=trace,hyper=off");

    assert_eq!(filter.widest(), Threshold::At(Level::Trace));
}

#[test]
fn the_widest_threshold_is_the_global_one_when_every_target_is_quieter() {
    let filter = Filter::from_directives("debug,hyper=off,rustls=warn");

    assert_eq!(filter.widest(), Threshold::At(Level::Debug));
}

#[test]
fn widest_and_narrowest_order_off_below_every_level() {
    let error = Threshold::At(Level::Error);

    assert_eq!(Threshold::Off.widest(error), error);
    assert_eq!(Threshold::Off.narrowest(error), Threshold::Off);
    assert_eq!(
        Threshold::At(Level::Trace).narrowest(Threshold::At(Level::Info)),
        Threshold::At(Level::Info)
    );
}

#[test]
fn a_threshold_of_off_admits_no_level_at_all() {
    assert!(!Threshold::Off.admits(Level::Error));
    assert!(Threshold::At(Level::Error).admits(Level::Error));
}
