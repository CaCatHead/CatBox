use std::collections::HashMap;
use std::ffi::{c_long, c_ulonglong};

use nix::libc::{SYS_accept, SYS_accept4, SYS_bind, SYS_clone, SYS_connect, SYS_execve, SYS_execveat, SYS_fork, SYS_getpeername, SYS_getsockname, SYS_getsockopt, SYS_listen, SYS_setsockopt, SYS_shutdown, SYS_socket, SYS_socketpair, SYS_vfork, user_regs_struct};
use nix::unistd::Pid;

type SyscallId = c_ulonglong;

/// 禁止系统调用
/// 允许有限次系统调用
#[derive(Debug, Clone)]
pub enum SyscallPerm {
  Forbid,
  FilterFn,
  Allow(i32),
}

/// 系统调用过滤器
/// 黑名单过滤，若不在映射内，则允许；否则，禁止或者允许有限次
#[derive(Debug, Clone)]
pub struct SyscallFilter {
  map: HashMap<SyscallId, SyscallPerm>,
}

impl SyscallPerm {
  fn forbid() -> Self {
    SyscallPerm::Forbid
  }

  fn allow(count: i32) -> Self {
    SyscallPerm::Allow(count)
  }
}

impl SyscallFilter {
  pub fn default() -> Self {
    let mut filter = SyscallFilter {
      map: HashMap::new()
    };
    // 禁用网络
    filter.forbid(SYS_socket)
      .forbid(SYS_socketpair)
      .forbid(SYS_setsockopt)
      .forbid(SYS_getsockopt)
      .forbid(SYS_getsockname)
      .forbid(SYS_getpeername)
      .forbid(SYS_bind)
      .forbid(SYS_listen)
      .forbid(SYS_accept)
      .forbid(SYS_accept4)
      .forbid(SYS_connect)
      .forbid(SYS_shutdown);
    // 禁用进程相关
    filter.allow(SYS_execve, 1)
      .allow(SYS_execveat, 1)
      .forbid(SYS_fork)
      .forbid(SYS_vfork)
      .forbid(SYS_clone);
    filter
  }

  pub fn forbid(self: &mut Self, id: c_long) -> &mut Self {
    self.map.insert(id as SyscallId, SyscallPerm::forbid());
    self
  }

  pub fn allow(self: &mut Self, id: c_long, count: i32) -> &mut Self {
    self.map.insert(id as SyscallId, SyscallPerm::allow(count));
    self
  }

  pub fn filter(self: &mut Self, pid: &Pid, regs: &user_regs_struct) -> bool {
    let syscall_id = regs.orig_rax;
    let entry = self.map.get(&syscall_id);
    if let Some(perm) = entry {
      match *perm {
        SyscallPerm::Forbid => false,
        SyscallPerm::FilterFn => false,
        SyscallPerm::Allow(mut count) => {
          if count == 0 {
            false
          } else {
            count -= 1;
            true
          }
        }
      }
    } else {
      true
    }
  }
}
