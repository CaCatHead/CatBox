use std::collections::hash_map::Entry::Occupied;
use std::collections::HashMap;
use std::ffi::{c_long, c_ulonglong};
use std::fmt::{Debug, Formatter};

use nix::libc::{
  user_regs_struct, SYS_accept, SYS_accept4, SYS_bind, SYS_clone, SYS_clone3,
  SYS_execve, SYS_execveat, SYS_fork, SYS_getpeername, SYS_getsockname, SYS_getsockopt, SYS_listen,
  SYS_setsockopt, SYS_shutdown, SYS_socketpair, SYS_vfork,
};
use nix::unistd::Pid;

use crate::CatBoxError;

type SyscallId = c_ulonglong;

/// Syscall permission
#[derive(Clone)]
pub enum SyscallPerm {
  /// Forbid all
  Forbid,
  /// Use a filter function to check whether it is ok
  FilterFn(fn(pid: &Pid, regs: &user_regs_struct) -> bool),
  /// Allow a few times
  Allow(i32),
}

/// Syscall filter
/// It is a black list filter, and it supports forbid syscall or allow a few times
#[derive(Debug, Clone)]
pub struct SyscallFilter {
  map: HashMap<SyscallId, SyscallPerm>,
}

/// Syscall filter preset category
#[derive(Debug, Copy, Clone)]
pub enum RestrictedSyscall {
  Net,
  Process,
  Thread,
}

impl SyscallFilter {
  /// Create an empty syscall filter
  pub fn new() -> Self {
    let filter = SyscallFilter {
      map: HashMap::new(),
    };
    filter
  }

  /// Create a default syscall filter with all the presets open
  pub fn default() -> Self {
    let mut filter = Self::new();
    filter
      .enable(RestrictedSyscall::Net)
      .enable(RestrictedSyscall::Process);
    filter
  }

  /// Enable preset
  pub fn enable(self: &mut Self, feature: RestrictedSyscall) -> &mut Self {
    match feature {
      RestrictedSyscall::Net => {
        self
          // .forbid(SYS_socket)
          .forbid(SYS_socketpair)
          .forbid(SYS_setsockopt)
          .forbid(SYS_getsockopt)
          .forbid(SYS_getsockname)
          .forbid(SYS_getpeername)
          .forbid(SYS_bind)
          .forbid(SYS_listen)
          .forbid(SYS_accept)
          .forbid(SYS_accept4)
          // .forbid(SYS_connect)
          .forbid(SYS_shutdown);
      }
      RestrictedSyscall::Process => {
        self
          .allow(SYS_execve, 1)
          .allow(SYS_execveat, 1)
          .forbid(SYS_fork)
          .forbid(SYS_vfork)
          .forbid(SYS_clone)
          .forbid(SYS_clone3);
      }
      RestrictedSyscall::Thread => {}
    };
    self
  }

  /// Try parsing presets string
  pub fn parse_presets(presets: Vec<String>) -> Result<Option<Self>, CatBoxError> {
    let mut filter = Self::new();
    let presets = presets
      .into_iter()
      .flat_map(|str| str.split(" ").map(str::to_owned).collect::<Vec<_>>())
      .map(|p| p.trim().to_ascii_lowercase())
      .filter(|p| p.len() > 0)
      .collect::<Vec<String>>();
    for preset in presets {
      match preset.as_str() {
        "none" => return Ok(None),
        "net" | "network" => {
          filter.enable(RestrictedSyscall::Net);
        }
        "process" => {
          filter.enable(RestrictedSyscall::Process);
        }
        "all" => {
          filter
            .enable(RestrictedSyscall::Net)
            .enable(RestrictedSyscall::Process);
        }
        _ => return Err(CatBoxError::cli("Parse ptrace syscall filter string fails")),
      };
    }
    Ok(Some(filter))
  }

  pub fn forbid(self: &mut Self, id: c_long) -> &mut Self {
    self.map.insert(id as SyscallId, SyscallPerm::forbid());
    self
  }

  pub fn add_fn(
    self: &mut Self,
    id: c_long,
    func: fn(pid: &Pid, regs: &user_regs_struct) -> bool,
  ) -> &mut Self {
    self
      .map
      .insert(id as SyscallId, SyscallPerm::FilterFn(func));
    self
  }

  pub fn allow(self: &mut Self, id: c_long, count: i32) -> &mut Self {
    self.map.insert(id as SyscallId, SyscallPerm::allow(count));
    self
  }

  pub fn filter(self: &mut Self, pid: &Pid, regs: &user_regs_struct) -> bool {
    let syscall_id = regs.orig_rax;
    let entry = self.map.entry(syscall_id);
    if let Occupied(mut entry) = entry {
      let perm = entry.get_mut();
      match perm {
        SyscallPerm::Forbid => false,
        SyscallPerm::FilterFn(func) => func(pid, regs),
        SyscallPerm::Allow(ref mut count) => {
          if *count == 0 {
            false
          } else {
            *count -= 1;
            true
          }
        }
      }
    } else {
      true
    }
  }
}

impl SyscallPerm {
  fn forbid() -> Self {
    SyscallPerm::Forbid
  }

  fn allow(count: i32) -> Self {
    SyscallPerm::Allow(count)
  }
}

impl Debug for SyscallPerm {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      SyscallPerm::Forbid => f.debug_struct("Forbid").finish(),
      SyscallPerm::FilterFn(_) => f.debug_struct("FilterFn").field("func", &"[func]").finish(),
      SyscallPerm::Allow(count) => f.debug_tuple("Allow").field(count).finish(),
    }
  }
}
