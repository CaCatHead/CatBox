# CatJ

[![CI](https://github.com/yjl9903/CatBox/actions/workflows/ci.yml/badge.svg)](https://github.com/yjl9903/CatBox/actions/workflows/ci.yml) [![](https://img.shields.io/crates/v/catj)](https://crates.io/crates/catj)

A light process isolation sandbox used for Competitive Programming contest.

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
catj compile ./fixtures/aplusb/source/ac.cpp -l cpp -o a.out
catj --stdin ./fixtures/aplusb/testcases/1.in --stdout ./logs/sub.out run -- ./a.out
```

## License

MIT License © 2023 [XLor](https://github.com/yjl9903)
