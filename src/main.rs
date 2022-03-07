use std::{
    fs::{read_to_string, File},
    io::{BufRead, BufReader},
    path::Path,
};
use std::io::{Error, ErrorKind};
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


fn list_disks() -> Vec<String> {
    let file = File::open("/proc/diskstats").unwrap();
    let mut file = BufReader::with_capacity(2048, file);
    let mut line = String::with_capacity(256);
    let mut disk_list: Vec<String> = Vec::new();
    while file.read_line(&mut line).unwrap() != 0 {
        let mut fields = line.split_whitespace();
        let name = nth!(fields, 2).unwrap();
        if name.contains("sr") || name.contains("scd") || name.contains("hdc") || disk_list.contains(&name.to_owned()) {
            continue;
        }
        if Path::new(&format!("/sys/block/{}/device", name.replace("/", "!"))).exists() {
            disk_list.push(name.to_owned());
        }
    }
    disk_list

}

fn get_disk_size(disk_name: &str) -> i64 {
    let sector_unit: i64 = read_to_string(&format!("/sys/block/{}/queue/logical_block_size", disk_name)).unwrap().trim().parse().unwrap();
    let sector_size: i64 = read_to_string(&format!("/sys/block/{}/size", disk_name)).unwrap().trim().parse().unwrap();
    sector_size * sector_unit
}

fn main() {
    let disks = list_disks();
    let total_size = disks.into_iter().map(|x| get_disk_size(&x)).fold(0, |acc, x| acc + x);
    println!("{:.1}", total_size as f64 / 1024.0 / 1024.0 / 1024.0);
}
