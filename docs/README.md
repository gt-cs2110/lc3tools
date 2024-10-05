# LC3Tools

LC3Tools is a modern set of tools to build code for and simulate the LC-3 system described in *Introduction to Computing* by Dr. Yale Patt and Dr. Sanjay Patel.

This project is a fork of the original LC3Tools ([repository](https://github.com/chiragsakhuja/lc3tools)) created by Chirag Sakhuja to better fit the instruction of Georgia Tech's CS 2110 course.

This project's aims are similar to that of the original, namely:

- Consistent cross-platform support (across Windows, macOS, and Linux)
- Consistent behavior across the GUI, command line tools, and other applications (though, this project does not maintain a CLI)
- Intuitive user interface
- Powerful testing API for unit tests and auto-graders
- Well-documented, simple, open-source code base

## References

- [Installing (and Uninstalling)](./INSTALL.md)
- [Building](./BUILDING.md)
- [Developing](./DEVELOPING.md)

## Project Structure

This project is built upon the [`ensemble` engine](https://github.com/endorpersand/lc3-ensemble), written in Rust. The frontend is written with Vue 3 and Electron Forge.
