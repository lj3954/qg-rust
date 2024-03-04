mod libosinfo;

use libosinfo::{OS, gather_osinfo};

fn main() {
    let distros = gather_osinfo().expect("Failed to gather OSinfo");
}

