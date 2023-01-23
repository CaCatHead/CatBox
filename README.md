# CatJ

[![CI](https://github.com/yjl9903/CatBox/actions/workflows/ci.yml/badge.svg)](https://github.com/yjl9903/CatBox/actions/workflows/ci.yml) [![](https://img.shields.io/crates/v/catj)](https://crates.io/crates/catj)

A light process isolation sandbox used for Competitive Programming contest.

## Features
 
+ [cgroups](https://man7.org/linux/man-pages/man7/cgroups.7.html): Record cpu and memory usage (may fall back to [getrusage](https://man7.org/linux/man-pages/man2/getrusage.2.html))
+ [mount](https://man7.org/linux/man-pages/man2/mount.2.html) and [chroot](https://man7.org/linux/man-pages/man2/chroot.2.html): Created an isolated file system
+ [setrlimit](https://man7.org/linux/man-pages/man2/getrlimit.2.html): Set resource limits (stack size)
+ [setuid](https://man7.org/linux/man-pages/man2/setuid.2.html) and [setgid](https://man7.org/linux/man-pages/man2/setuid.2.html): Run submission under another user and group
+ [ptrace](https://man7.org/linux/man-pages/man2/ptrace.2.html): Filter submission syscall

## Installation

```bash
# Install using cargo
$ cargo install catj
$ catj --version

# Init cgroup for current user
$ ./init.sh $USER
```

## Usage

```bash
# Compile C++ source code
$ catj compile ./fixtures/aplusb/source/ac.cpp -o a.out

# Run a.out
$ catj run --stdin ./fixtures/aplusb/testcases/1.in -- ./a.out

# Generate report
$ catj --report run --stdin ./fixtures/aplusb/testcases/1.in --stdout ./sub.out -- ./a.out
```

## License

MIT License © 2023 [XLor](https://github.com/yjl9903)
