#[cfg(test)]
use builder::OptionDefinition;

mod actions;
mod builder;
mod errors;
mod machine;
mod node;
pub mod runner;
mod shared;
mod transition;

#[test]
fn it_should_select_the_default_command_when_using_no_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec![]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_select_the_default_command_when_using_mandatory_positional_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap()
        .add_positional(true, "bar").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo", "bar"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_select_commands_by_their_path() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    cli_builder.add_command()
        .add_path(vec!["bar"]);

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));

    let result = runner::run_machine(&machine, &vec!["bar"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_select_commands_by_their_required_positional_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec![]).unwrap();
    assert_eq!(result.selected_index, Some(0));

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_select_options_by_their_simple_options() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], ..Default::default()}).unwrap();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["-y".to_string()], ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    println!("{:?}", machine);

    let result = runner::run_machine(&machine, &vec!["-x"]).unwrap();
    assert_eq!(result.selected_index, Some(0));

    let result = runner::run_machine(&machine, &vec!["-y"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_allow_options_to_precede_the_command_paths() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"])
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], ..Default::default()}).unwrap();

    cli_builder.add_command()
        .add_path(vec!["bar"])
        .add_option(OptionDefinition {name_set: vec!["-y".to_string()], ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["-x", "foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));

    let result = runner::run_machine(&machine, &vec!["-y", "bar"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_select_commands_by_their_complex_values() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], arity: 1, ..Default::default()}).unwrap();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["-y".to_string()], arity: 1, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["-x", "foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));

    let result = runner::run_machine(&machine, &vec!["-y", "bar"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_prefer_longer_paths_over_mandatory_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_prefer_longer_paths_over_mandatory_arguments_reversed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_prefer_longer_paths_over_mandatory_arguments_prefixed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["prfx", "foo"]);

    cli_builder.add_command()
        .add_path(vec!["prfx"])
        .add_positional(true, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["prfx", "foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_prefer_longer_paths_over_optional_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    cli_builder.add_command()
        .make_default()
        .add_positional(false, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_prefer_longer_paths_over_optional_arguments_reversed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(false, "foo").unwrap();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_prefer_longer_paths_over_optional_arguments_prefixed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["prfx", "foo"]);

    cli_builder.add_command()
        .add_path(vec!["prfx"])
        .add_positional(false, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["prfx", "foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_prefer_required_arguments_over_optional_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap();

    cli_builder.add_command()
        .make_default()
        .add_positional(false, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_prefer_required_arguments_over_optional_arguments_reversed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(false, "foo").unwrap();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_fallback_from_path_to_required_arguments_if_needed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["bar"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_fallback_from_path_to_required_arguments_if_needed_reverse() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["bar"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_fallback_from_path_to_required_arguments_if_needed_prefixed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["prfx", "foo"]);

    cli_builder.add_command()
        .add_path(vec!["prfx"])
        .add_positional(true, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["prfx", "bar"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_fallback_from_path_to_optional_arguments_if_needed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    cli_builder.add_command()
        .make_default()
        .add_positional(false, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["bar"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_fallback_from_path_to_optional_arguments_if_needed_reverse() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(false, "foo").unwrap();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["bar"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_fallback_from_path_to_optional_arguments_if_needed_prefixed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["prfx", "foo"]);

    cli_builder.add_command()
        .add_path(vec!["prfx"])
        .add_positional(false, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["prfx", "bar"]).unwrap();
    assert_eq!(result.selected_index, Some(1));
}

#[test]
fn it_should_extract_booleans_from_simple_options() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["-x"]).unwrap();
    assert_eq!(result.options, vec![("-x".to_string(), runner::OptionValue::Bool(true))]);
}

#[test]
fn it_should_extract_booleans_from_batch_options() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], ..Default::default()}).unwrap()
        .add_option(OptionDefinition {name_set: vec!["-y".to_string()], ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["-xy"]).unwrap();
    assert_eq!(result.options, vec![
        ("-x".to_string(), runner::OptionValue::Bool(true)),
        ("-y".to_string(), runner::OptionValue::Bool(true)),
    ]);
}

#[test]
fn it_should_invert_booleans_when_using_no() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["--foo".to_string()], ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--no-foo"]).unwrap();
    assert_eq!(result.options, vec![("--foo".to_string(), runner::OptionValue::Bool(false))]);
}

#[test]
fn it_should_extract_strings_from_complex_options() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], arity: 1, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["-x", "foo"]).unwrap();
    assert_eq!(result.options, vec![("-x".to_string(), runner::OptionValue::String("foo".to_string()))]);
}

#[test]
fn it_should_extract_strings_from_complex_options_with_equals() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["--foo".to_string()], arity: 1, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--foo=foo"]).unwrap();
    assert_eq!(result.options, vec![("--foo".to_string(), runner::OptionValue::String("foo".to_string()))]);
}

#[test]
fn it_shouldnt_consider_dash_as_an_option() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["--foo".to_string()], arity: 1, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--foo", "-"]).unwrap();
    assert_eq!(result.options, vec![("--foo".to_string(), runner::OptionValue::String("-".to_string()))]);
}

#[test]
fn it_should_extract_arrays_from_complex_options() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["--foo".to_string()], arity: 1, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--foo", "bar", "--foo", "baz"]).unwrap();
    assert_eq!(result.options, vec![
        ("--foo".to_string(), runner::OptionValue::String("bar".to_string())),
        ("--foo".to_string(), runner::OptionValue::String("baz".to_string())),
    ]);
}

#[test]
fn it_should_extract_arrays_from_complex_options_equal() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["--foo".to_string()], arity: 1, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--foo=bar", "--foo=baz"]).unwrap();
    assert_eq!(result.options, vec![
        ("--foo".to_string(), runner::OptionValue::String("bar".to_string())),
        ("--foo".to_string(), runner::OptionValue::String("baz".to_string())),
    ]);
}

#[test]
fn it_should_extract_arrays_from_complex_options_mixed() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["--foo".to_string()], arity: 1, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--foo", "bar", "--foo=baz"]).unwrap();
    assert_eq!(result.options, vec![
        ("--foo".to_string(), runner::OptionValue::String("bar".to_string())),
        ("--foo".to_string(), runner::OptionValue::String("baz".to_string())),
    ]);
}

#[test]
fn it_should_support_rest_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_rest("rest").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo", "bar", "baz"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Rest("foo".to_string()),
        runner::Positional::Rest("bar".to_string()),
        runner::Positional::Rest("baz".to_string()),
    ]);
}

#[test]
fn it_should_support_rest_arguments_followed_by_mandatory_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_rest("rest").unwrap()
        .add_positional(true, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["src1", "src2", "src3", "dest"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Rest("src1".to_string()),
        runner::Positional::Rest("src2".to_string()),
        runner::Positional::Rest("src3".to_string()),
        runner::Positional::Required("dest".to_string()),
    ]);
}

#[test]
fn it_should_support_rest_arguments_between_mandatory_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap()
        .add_rest("rest").unwrap()
        .add_positional(true, "bar").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo", "src1", "src2", "src3", "dest"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Required("foo".to_string()),
        runner::Positional::Rest("src1".to_string()),
        runner::Positional::Rest("src2".to_string()),
        runner::Positional::Rest("src3".to_string()),
        runner::Positional::Required("dest".to_string()),
    ]);
}

#[test]
fn it_should_support_option_arguments_in_between_rest_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["--foo".to_string()], ..Default::default()}).unwrap()
        .add_option(OptionDefinition {name_set: vec!["--bar".to_string()], arity: 1, ..Default::default()}).unwrap()
        .add_rest("rest").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["src1", "--foo", "src2", "--bar", "baz", "src3"]).unwrap();

    assert_eq!(result.options, vec![
        ("--foo".to_string(), runner::OptionValue::Bool(true)),
        ("--bar".to_string(), runner::OptionValue::String("baz".to_string())),
    ]);

    assert_eq!(result.positionals, vec![
        runner::Positional::Rest("src1".to_string()),
        runner::Positional::Rest("src2".to_string()),
        runner::Positional::Rest("src3".to_string()),
    ]);
}

#[test]
fn it_should_ignore_options_when_they_follow_the_dash_dash_separator() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"])
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], ..Default::default()}).unwrap()
        .add_positional(false, "foo").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo", "--", "-x"]).unwrap();

    assert_eq!(result.options, vec![
        // Must be empty
    ]);

    assert_eq!(result.positionals, vec![
        runner::Positional::Optional("-x".to_string()),
    ]);
}

#[test]
fn it_should_ignore_options_when_they_appear_after_a_required_positional_from_a_proxy() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"])
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], ..Default::default()}).unwrap()
        .add_positional(true, "foo").unwrap()
        .add_positional(true, "bar").unwrap()
        .add_proxy("proxy").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo", "pos1", "-x", "pos2", "proxy"]).unwrap();

    assert_eq!(result.options, vec![
        ("-x".to_string(), runner::OptionValue::Bool(true)),
    ]);

    assert_eq!(result.positionals, vec![
        runner::Positional::Required("pos1".to_string()),
        runner::Positional::Required("pos2".to_string()),
        runner::Positional::Rest("proxy".to_string()),
    ]);
}

#[test]
fn it_should_ignore_options_when_they_appear_in_a_proxy_extra() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"])
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], ..Default::default()}).unwrap()
        .add_proxy("proxy").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo", "-x"]).unwrap();

    assert_eq!(result.options, vec![
        // Must be empty
    ]);

    assert_eq!(result.positionals, vec![
        runner::Positional::Rest("-x".to_string()),
    ]);
}

#[test]
fn it_should_prefer_exact_commands_over_empty_proxies() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .add_path(vec!["foo"]);

    cli_builder.add_command()
        .add_path(vec!["foo"])
        .add_positional(true, "foo").unwrap()
        .add_proxy("proxy").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.selected_index, Some(0));
}

#[test]
fn it_should_aggregate_the_options_as_they_are_found() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["-x".to_string()], ..Default::default()}).unwrap()
        .add_option(OptionDefinition {name_set: vec!["-y".to_string()], ..Default::default()}).unwrap()
        .add_option(OptionDefinition {name_set: vec!["-z".to_string()], ..Default::default()}).unwrap()
        .add_option(OptionDefinition {name_set: vec!["-u".to_string()], arity: 1, ..Default::default()}).unwrap()
        .add_option(OptionDefinition {name_set: vec!["-v".to_string()], arity: 1, ..Default::default()}).unwrap()
        .add_option(OptionDefinition {name_set: vec!["-w".to_string()], arity: 1, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["-x", "-u", "foo", "-y", "-v", "bar", "-y"]).unwrap();
    assert_eq!(result.options, vec![
        ("-x".to_string(), runner::OptionValue::Bool(true)),
        ("-u".to_string(), runner::OptionValue::String("foo".to_string())),
        ("-y".to_string(), runner::OptionValue::Bool(true)),
        ("-v".to_string(), runner::OptionValue::String("bar".to_string())),
        ("-y".to_string(), runner::OptionValue::Bool(true)),
    ]);

    let result = runner::run_machine(&machine, &vec!["-z", "-y", "-x"]).unwrap();
    assert_eq!(result.options, vec![
        ("-z".to_string(), runner::OptionValue::Bool(true)),
        ("-y".to_string(), runner::OptionValue::Bool(true)),
        ("-x".to_string(), runner::OptionValue::Bool(true)),
    ]);
}

#[test]
fn it_should_aggregate_the_mandatory_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap()
        .add_positional(true, "bar").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo", "bar"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Required("foo".to_string()),
        runner::Positional::Required("bar".to_string()),
    ]);
}

#[test]
fn it_should_aggregate_the_optional_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(false, "foo").unwrap()
        .add_positional(false, "bar").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo", "bar"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Optional("foo".to_string()),
        runner::Positional::Optional("bar".to_string()),
    ]);
}

#[test]
fn it_should_accept_as_few_optional_arguments_as_possible() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(false, "foo").unwrap()
        .add_positional(false, "bar").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec![]).unwrap();
    assert_eq!(result.positionals, vec![]);

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Optional("foo".to_string()),
    ]);
}

#[test]
fn it_should_accept_a_mix_of_mandatory_and_optional_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_positional(true, "foo").unwrap()
        .add_positional(false, "bar").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Required("foo".to_string()),
    ]);

    let result = runner::run_machine(&machine, &vec!["foo", "bar"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Required("foo".to_string()),
        runner::Positional::Optional("bar".to_string()),
    ]);
}

#[test]
fn it_should_accept_any_option_as_positional_argument_when_proxies_are_enabled() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_proxy("proxy").unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--foo", "--bar"]).unwrap();
    assert_eq!(result.positionals, vec![
        runner::Positional::Rest("--foo".to_string()),
        runner::Positional::Rest("--bar".to_string()),
    ]);
}

#[cfg(test)]
fn check_syntax_error<T>(err: Result<T, errors::Error>, str: &str) {
    match err {
        Err(errors::Error::UnknownSyntax(s)) => assert!(s.starts_with(str), "Expected '{}' to start with '{}'", s, str),
        _ => panic!("Expected an UnknownSyntax error"),
    }
}

#[test]
fn it_should_throw_acceptable_errors_when_passing_an_extraneous_option() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--foo"]);
    check_syntax_error(result, "Unsupported option name (\"--foo\")");
}

#[test]
fn it_should_throw_acceptable_errors_when_passing_extraneous_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["foo"]);
    check_syntax_error(result, "Extraneous positional argument (\"foo\")");
}

#[test]
fn it_should_throw_acceptable_errors_when_writing_invalid_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["-%#@$%#()@"]);
    check_syntax_error(result, "Invalid option name (\"-%#@$%#()@\")");
}

#[test]
fn it_should_throw_acceptable_errors_when_writing_bound_boolean_arguments() {
    let mut cli_builder = builder::CliBuilder::new();

    cli_builder.add_command()
        .make_default()
        .add_option(OptionDefinition {name_set: vec!["--foo".to_string()], allow_binding: false, ..Default::default()}).unwrap();

    let machine = cli_builder.compile();

    let result = runner::run_machine(&machine, &vec!["--foo=bar"]);
    check_syntax_error(result, "Invalid option name (\"--foo=bar\")");
}
