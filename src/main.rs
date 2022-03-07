use std::io::{Error, ErrorKind};
use std::{
    fs::{read_to_string, File},
    io::{self, BufRead, BufReader},
    path::Path,
};
macro_rules! nth {
    ($a:expr, $b:expr) => {
        match $a.nth($b) {
            Some(val) => Ok(val),
            None => Err(Error::new(
                ErrorKind::Other,
                "The fields asked for does not exists",
            )),
        }
    };
}

fn list_disks() -> io::Result<Vec<String>> {
    File::open("/proc/diskstats").and_then(|file_in| {
        let file_reader = BufReader::with_capacity(2048, file_in);
        Ok(file_reader
            .lines()
            .filter_map(|line_in| match line_in {
                Ok(line) => {
                    let mut fields = line.split_whitespace();
                    nth!(fields, 2)
                        .and_then(|name| match name {
                            name if name.contains("sr")
                                || name.contains("scd")
                                || name.contains("hdc") =>
                            {
                                Ok(None)
                            }
                            name if Path::new(&format!(
                                "/sys/block/{}/device",
                                name.replace("/", "!")
                            ))
                            .exists() =>
                            {
                                Ok(Some(name.to_owned()))
                            }
                            _ => Ok(None),
                        })
                        .unwrap()
                }
                _ => None,
            })
            .collect::<Vec<String>>())
    })
}

fn read_to_i64(filename: &str) -> Result<i64, String> {
    match read_to_string(filename).and_then(|content| Ok(content.trim().parse::<i64>())) {
        Ok(Ok(intdata)) => Ok(intdata),
        _ => Err("Failed".to_owned()),
    }
}

fn get_disk_size(disk_name: &str) -> Result<i64, String> {
    let sector_unit_file = format!("/sys/block/{}/queue/logical_block_size", disk_name);
    let sector_size_file = format!("/sys/block/{}/size", disk_name);
    read_to_i64(&sector_size_file).and_then(|sector_size| {
        read_to_i64(&sector_unit_file).and_then(|sector_unit| Ok(sector_size * sector_unit))
    })
}

fn main() {
    match list_disks().and_then(|disks| {
        Ok(disks
            .into_iter()
            .map(|x| match get_disk_size(&x) {
                Ok(val) => val,
                Err(_) => 0,
            })
            .fold(0, |acc, x| acc + x))
    }) {
        Ok(total_size) => println!(
            "Total Size: {:.1} Gib",
            total_size as f64 / 1024.0 / 1024.0 / 1024.0
        ),
        _ => println!("Failed to get disk size."),
    }
}
