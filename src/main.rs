mod utils;
mod basic_distros;

use utils::{Distro, collect_distros};


fn main() {
    let distros = collect_distros();
}

