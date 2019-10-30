mod validated_mapfile;
use ddrescue_mapfile::parse_mapfile;
use iso9660::{DirectoryEntry, DirectoryEntryHeader, ISODirectory, ISOFile, ISO9660};
use nom::IResult;
use std::fs::File;
use std::io::Read;
use validated_mapfile::BadSectors;

fn mapfile(file: &str) -> BadSectors {
    let mut mapfile = File::open(file).unwrap();
    let mut data = String::new();
    mapfile.read_to_string(&mut data).unwrap();
    println!("{:?}", data);
    parse_mapfile(&data).unwrap().1.into()
}

impl BadSectors {
    fn test_bad(&self, file: &DirectoryEntryHeader, identifier: &str) {
        let start = file.extent_loc as usize * 2048;
        let end = file.extent_length as usize + start;
        let bad = self.contains_bad_sector(start, end);
        if let Err((bad_start, bad_end)) = bad {
            println!(
                "Bad entry {:?} : {:?} to {:?} is bad",
                identifier, bad_start, bad_end
            );
        }
    }
}
use clap::{App, Arg, ArgGroup};

fn main() {
    let m = App::new("dd_iso_check")
        .version("1.0")
        .about("Check which files in an iso are bad given a ddrescue mapfile.")
        .author("Ruben Lapauw")
        .arg(
            Arg::with_name("prefix")
                .short("p")
                .long("prefix")
                .help("File prefix to use the default extensions .iso and .map")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("isofile")
                .short("i")
                .long("iso")
                .help("Which isofile to check")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("mapfile")
                .short("m")
                .long("map")
                .help("Which mapfile to use")
                .takes_value(true),
        )
        .group(
            ArgGroup::with_name("iso")
                .args(&["prefix", "isofile"])
                .required(true),
        )
        .group(
            ArgGroup::with_name("map")
                .args(&["prefix", "mapfile"])
                .required(true),
        )
        .get_matches();
    let (iso, map) = if let Some(prefix) = m.value_of("prefix") {
        (prefix.to_owned() + ".iso", prefix.to_owned() + ".map")
    } else {
        (
            m.value_of("iso").unwrap().to_owned(),
            m.value_of("mapfile").unwrap().to_owned(),
        )
    };
    let bad_sectors = mapfile(&map);
    let file = File::open(&iso).unwrap();
    let fs = ISO9660::new(file).unwrap();
    let mut queued = vec![fs.root];
    while let Some(dir) = queued.pop() {
        for c in dir.contents() {
            let entry = match c {
                Err(e) => panic!("Error in {:?}: {:?}", dir, e),
                Ok(ok) => ok,
            };
            bad_sectors.test_bad(entry.header(), entry.identifier());
            if let DirectoryEntry::Directory(dir) = entry {
                if dir.identifier == "." || dir.identifier == ".." {
                    continue;
                }
                queued.push(dir);
            }
        }
    }
}
