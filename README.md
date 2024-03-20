This is a re-implementation of Quickget from the Quickemu project (https://github.com/quickemu-project/quickemu) in Rust.

This branch focuses on replicating the behaviour of the original bash script as effectively as possible. 

Performance will be prioritized. For example, checksums will be fetched during, rather than before, the file download, which can save a significant amount of time depending on connection.
Releases & Editions are also fetched from within the same function in the case that they're dynamic, which means that it's possible that only one request needs to be made. 
After all, almost all of the time waiting is caused by requests to web servers.

It is very much **WIP**. The majority of features are not implemented, and there are many major bugs. 

Currently, downloads are essentially fully implemented, as well as checksum verification. 
The next step is to implement VM creation. 

Comments will be added throughout the file distros are added in, in order to make implementation of new OSes easier,
hopefully including for people from the original quickemu project who may not understand any Rust. 

## Compatibility

This program by itself is capable of being compiled and run on nearly any system. However, the VMs it creates rely on Quickemu to be of any use, which is only available for Unix-like systems such as GNU/Linux.
However, the `download-iso` parameter (TO BE IMPLEMENTED) will be entirely usable on any system. 

## Instructions

1. Install the Rust language. Many distros package it, but the recommended method is to use the Rustup script.
2. Clone the repository: `git clone https://github.com/lj3954/qg-rust`
3. Build the project using `cargo build`
4. The compiled binary will be located within the "target" folder.
