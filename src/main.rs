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

fn is_not_cdrom(disk_name: &str) -> bool {
    !(disk_name.contains("hdc") || disk_name.contains("sr") || disk_name.contains("scd"))
}

fn is_physical_device(disk_name: &str) -> bool {
    let device_path = Path::new(&format!(
        "/sys/block/{}/device",
        disk_name.replace("/", "!")
    ));
    device_path.exists() && is_not_cdrom(disk_name)
}

fn list_disks() -> io::Result<Vec<String>> {
    File::open("/proc/diskstats").and_then(|file_in| {
        let file_reader = BufReader::with_capacity(2048, file_in);
        Ok(file_reader
            .lines()
            .filter_map(|line_in| {
                line_in
                    .and_then(|line| {
                        let mut fields = line.split_whitespace();
                        nth!(fields, 2).and_then(|name| {
                            Ok({
                                if is_physical_device(name) {
                                    Some(name.to_owned())
                                } else {
                                    None
                                }
                            })
                        })
                    })
                    .unwrap_or(None)
            })
            .collect::<Vec<String>>())
    })
}

fn read_to_i64(filename: &str) -> io::Result<i64> {
    read_to_string(filename).and_then(|content| Ok(content.trim().parse::<i64>().unwrap_or(0)))
}

fn get_disk_size(disk_name: &str) -> io::Result<i64> {
    let sector_unit_file = format!("/sys/block/{}/queue/logical_block_size", disk_name);
    let sector_size_file = format!("/sys/block/{}/size", disk_name);
    read_to_i64(&sector_size_file).and_then(|sector_size| {
        read_to_i64(&sector_unit_file).and_then(|sector_unit| Ok(sector_size * sector_unit))
    })
}

fn get_disk_total_size() -> io::Result<i64> {
    list_disks().and_then(|disks| {
        Ok(disks
            .into_iter()
            .map(|x| get_disk_size(&x).unwrap_or(0))
            .fold(0, |acc, x| acc + x))
    })
}

fn main() {
    match get_disk_total_size() {
        Ok(total_size) => println!("{}", total_size),
        _ => println!("Failed to get disk size.")
    }
}
