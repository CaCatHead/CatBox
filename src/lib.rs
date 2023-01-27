//! A light process isolation sandbox used for Competitive Programming contest.
//!
//! ## Features
//!  
//! + [cgroups](https://man7.org/linux/man-pages/man7/cgroups.7.html): Record cpu and memory usage (may fall back to [getrusage](https://man7.org/linux/man-pages/man2/getrusage.2.html))
//! + [mount](https://man7.org/linux/man-pages/man2/mount.2.html) and [chroot](https://man7.org/linux/man-pages/man2/chroot.2.html): Created an isolated file system
//! + [setrlimit](https://man7.org/linux/man-pages/man2/getrlimit.2.html): Set resource limits (cpu, address size, stack size, file size)
//! + [setuid](https://man7.org/linux/man-pages/man2/setuid.2.html) and [setgid](https://man7.org/linux/man-pages/man2/setuid.2.html): Run submission under another user and group
//! + [ptrace](https://man7.org/linux/man-pages/man2/ptrace.2.html): Filter submission syscall
//!
//! > **Note**
//! >
//! > To enable all of above features, it is highly recommended to use it under the **root** user, otherwise it may fall back automatically.
//!
//! ## Usage
//! ```bash
//! # Compile C++ source code
//! $ catj compile ./fixtures/aplusb/source/ac.cpp -o a.out
//!
//! # Run a.out
//! $ catj run --stdin ./fixtures/aplusb/testcases/1.in --read . -- ./a.out
//! 2
//!
//! # Generate report
//! $ catj --report run --stdin ./fixtures/aplusb/testcases/1.in --stdout ./sub.out --read . -- ./a.out
//! # or
//! $ catj -r run -i ./fixtures/aplusb/testcases/1.in -o ./sub.out -R . -- ./a.out
//! Status     0
//! Signal     ✓
//! Time       1 ms
//! Time user  1 ms
//! Time sys   0 ms
//! Memory     0 KB
//! ```

pub use catbox::run;
pub use context::{CatBox, CatBoxBuilder, CatBoxOption, CatBoxOptionBuilder, CatBoxResult};
pub use error::CatBoxError;
pub use syscall::{RestrictedSyscall, SyscallFilter};

mod catbox;
mod cgroup;
pub mod context;
mod error;
mod syscall;
mod utils;
