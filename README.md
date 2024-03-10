This is a re-implementation of Quickget from the Quickemu project (https://github.com/quickemu-project/quickemu) in Rust.

This branch focuses on replicating the behaviour of the original bash script as effectively as possible. 

It is very much **WIP**. The majority of features are not implemented, and there are many major bugs. 

## Compatibility

This program by itself is capable of being compiled and run on nearly any system. However, the VMs it creates rely on Quickemu to be of any use, which is only available for Unix-like systems such as GNU/Linux.
However, the `download-iso` parameter (TO BE IMPLEMENTED) will be entirely usable on any system. 

## Instructions

1. Install the Rust language. Many distros package it, but the recommended method is to use the Rustup script.
2. Clone the repository: `git clone https://github.com/lj3954/qg-rust`
3. Build the project using `cargo build`
4. The compiled binary will be located within the "target" folder.
