use super::data::*;
use nom::{
    character::complete::{digit1, hex_digit1, line_ending, not_line_ending, space1},
    bytes::complete::tag,
    delimited, do_parse, eof, many0, map, map_res, named, opt, preceded,
    return_error, separated_list, switch, take, value,
};

pub struct ParseError;

fn from_hex(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 16)
}
fn from_dec(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 10)
}

named!(dec_u64<&str, u64>, map_res!(digit1, from_dec));
named!(hex_u64<&str, u64>, preceded!(tag("0x"), map_res!(hex_digit1, from_hex)));
named!(address<&str, Address>, map!(hex_u64, Address));
named!(size<&str, Size>, map!(hex_u64, Size));
named!(pass<&str, Pass>, map!(dec_u64, Pass));
named!(current_status<&str, CurrentStatus>, switch!(take!(1),
    "?" => value!(CurrentStatus::CopyNonTriedBlock) |
    "*" => value!(CurrentStatus::TrimmingBlock) |
    "/" => value!(CurrentStatus::ScrapingBlock) |
    "-" => value!(CurrentStatus::RetryBadSector) |
    "F" => value!(CurrentStatus::Filling) |
    "G" => value!(CurrentStatus::Approximate) |
    "+" => value!(CurrentStatus::Finished)
));
named!(block_status<&str, BlockStatus>, switch!(take!(1),
    "?" => value!(BlockStatus::Untried) |
    "*" => value!(BlockStatus::NonTrimmed) |
    "/" => value!(BlockStatus::NonScraped) |
    "-" => value!(BlockStatus::BadSector) |
    "+" => value!(BlockStatus::Finished)
));

named!(current_state<&str, CurrentState>, do_parse!(
    current_pos:    address >>
                    space1 >>
    current_status: current_status >>
    current_pass:   opt!(preceded!(space1, pass)) >>
    (CurrentState{current_pos, current_status, current_pass})
));
named!(block<&str, Block>, do_parse!(
    pos:            address >>
                    space1 >>
    size:           size >>
                    space1 >>
    status:         block_status >>
    (Block{pos, size, status})
));

named!(comment<&str, &str>, delimited!(tag("#"), not_line_ending, line_ending));
named!(comment_lines<&str, ()>, value!((), many0!(comment)));
named!(pub parse_mapfile<&str, MapFile>, do_parse!(
                    comment_lines >>
    current_state:  current_state >>
                    line_ending >>
                    comment_lines >>
    blocks:         separated_list!(line_ending, block) >>
                    opt!(line_ending) >>
                    eof!() >>
    (
        MapFile {
        current_state,
        blocks,
    })
));

#[cfg(test)]
mod tests {
    use super::*;
    use nom::multi::many0;

    #[test]
    fn test_basic() {
        assert_eq!(comment("# comment\r\n"), Ok(("", " comment".into())));
        assert_eq!(hex_u64("0xdEaDbEeF "), Ok((" ", 0xdeadbeef)));
        assert_eq!(address("0xdEaDbEeF;"), Ok((";", Address(0xdeadbeef))));
        assert_eq!(size("0xdEaDbEeF;"), Ok((";", Size(0xdeadbeef))));
        assert_eq!(dec_u64("5;"), Ok((";", 5)));
        assert_eq!(pass("5;"), Ok((";", Pass(5))));
    }

    #[test]
    fn test_block_status() {
        use BlockStatus::*;
        assert_eq!(
            many0(block_status)("?*/-+;"),
            Ok((
                ";",
                vec![Untried, NonTrimmed, NonScraped, BadSector, Finished,]
            ))
        );
    }

    #[test]
    fn test_current_status() {
        use CurrentStatus::*;
        assert_eq!(
            many0(current_status)("?*/-FG+;"),
            Ok((
                ";",
                vec![
                    CopyNonTriedBlock,
                    TrimmingBlock,
                    ScrapingBlock,
                    RetryBadSector,
                    Filling,
                    Approximate,
                    Finished,
                ]
            ))
        );
    }

    #[test]
    fn test_current_state() {
        assert_eq!(
            current_state("0x24F35400     +\r\n"),
            Ok((
                "\r\n",
                CurrentState {
                    current_pos: Address(0x24f35400),
                    current_status: CurrentStatus::Finished,
                    current_pass: None,
                }
            ))
        );
        assert_eq!(
            current_state("0x24F35400     +   1\r\n"),
            Ok((
                "\r\n",
                CurrentState {
                    current_pos: Address(0x24f35400),
                    current_status: CurrentStatus::Finished,
                    current_pass: Some(Pass(1)),
                }
            ))
        );
    }

    #[test]
    fn test_block() {
        assert_eq!(
            block("0x00000001  0x2237B000  +;"),
            Ok((
                ";",
                Block {
                    pos: Address(0x1),
                    size: Size(0x2237B000),
                    status: BlockStatus::Finished,
                }
            ))
        );
    }

    #[test]
    fn test_file() {
        assert_eq!(
            parse_mapfile(
                "# Rescue Logfile.
# current_pos  current_status
0x24F35400     +
#      pos        size  status
0x00000000  0x2237B000  +
0x2237B000  0x02BBA800  -"
            ),
            Ok((
                "",
                MapFile {
                    current_state: CurrentState {
                        current_pos: Address(0x24f35400),
                        current_status: CurrentStatus::Finished,
                        current_pass: None,
                    },
                    blocks: vec![
                        Block {
                            pos: Address(0x0),
                            size: Size(0x2237B000),
                            status: BlockStatus::Finished,
                        },
                        Block {
                            pos: Address(0x2237B000),
                            size: Size(0x02BBA800),
                            status: BlockStatus::BadSector,
                        },
                    ],
                },
            )),
        );
    }
    #[test]
    fn test_file2() {
        assert_eq!(
            parse_mapfile(
                "# Rescue Logfile.
# current_pos  current_status
0x24F35400     +
#      pos        size  status
0x00000000  0x2237B000  +
0x2237B000  0x02BBA800  -
"
            ),
            Ok((
                "",
                MapFile {
                    current_state: CurrentState {
                        current_pos: Address(0x24f35400),
                        current_status: CurrentStatus::Finished,
                        current_pass: None,
                    },
                    blocks: vec![
                        Block {
                            pos: Address(0x0),
                            size: Size(0x2237B000),
                            status: BlockStatus::Finished,
                        },
                        Block {
                            pos: Address(0x2237B000),
                            size: Size(0x02BBA800),
                            status: BlockStatus::BadSector,
                        },
                    ],
                },
            )),
        );
    }
    #[test]
    fn test_mapfile_eof() {
        assert!(parse_mapfile(
            "# Rescue Logfile.
# current_pos  current_status
0x24F35400     +
#      pos        size  status
0x00000000  0x2237B000  +
0x2237B000  0x02BBA800  -;"
        )
        .is_err());
    }
}
