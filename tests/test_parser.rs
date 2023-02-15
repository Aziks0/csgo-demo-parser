use const_format::concatcp;
use std::fs::File;

use csgo_demo_parser::DemoParser;

const DATA_TESTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data");

#[test]
fn valid_demo_matchmaking_2022() {
    let mut demo_file =
        File::open(concatcp!(DATA_TESTS_DIR, "/matchmaking_2022_inferno.dem")).unwrap();
    let mut parser = DemoParser::try_new(&mut demo_file).unwrap();

    assert_eq!(parser.header().map_name(), &String::from("de_inferno"));

    let mut packet_number = 0;
    while parser.parse_next_packet().unwrap().is_some() {
        packet_number += 1;
    }

    assert_eq!(packet_number, 87191);
}

#[test]
fn valid_demo_hltv_2022() {
    let mut demo_file = File::open(concatcp!(
        DATA_TESTS_DIR,
        "/faze_vs_liquid_blast_premier_world_final_2022_anubis.dem"
    ))
    .unwrap();
    let mut parser = DemoParser::try_new(&mut demo_file).unwrap();

    assert_eq!(parser.header().map_name(), &String::from("de_anubis"));

    let mut packet_number = 0;
    while parser.parse_next_packet().unwrap().is_some() {
        packet_number += 1;
    }

    assert_eq!(packet_number, 426547);
}

#[test]
fn valid_demo_hltv_2019() {
    let mut demo_file = File::open(concatcp!(
        DATA_TESTS_DIR,
        "/ence_vs_astralis_iem_2019_inferno.dem"
    ))
    .unwrap();
    let mut parser = DemoParser::try_new(&mut demo_file).unwrap();

    assert_eq!(parser.header().map_name(), &String::from("de_inferno"));

    let mut packet_number = 0;
    while parser.parse_next_packet().unwrap().is_some() {
        packet_number += 1;
    }

    assert_eq!(packet_number, 281844);
}

#[test]
fn valid_demo_hltv_2018() {
    let mut demo_file = File::open(concatcp!(
        DATA_TESTS_DIR,
        "/natus_vincere_vs_astralis_faceit_major_2018_nuke.dem"
    ))
    .unwrap();
    let mut parser = DemoParser::try_new(&mut demo_file).unwrap();

    assert_eq!(parser.header().map_name(), &String::from("de_nuke"));

    let mut packet_number = 0;
    while parser.parse_next_packet().unwrap().is_some() {
        packet_number += 1;
    }

    assert_eq!(packet_number, 377764);
}
