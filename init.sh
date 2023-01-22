#!/bin/bash

if [ "$(id -u)" -ne 0 ]; then
    echo "Please re-run init.sh as root."
    exit 1
fi

catj_user="${1:-catj}"

echo "Register cgroup for ${catj_user}..."

ALL_SUBSYSTEMS=("cpu" "cpuacct" "memory" "pids")

for subsystem in "${ALL_SUBSYSTEMS[@]}"; do
  subsystem_dir="/sys/fs/cgroup/${catj_user}/${subsystem}/"
  mkdir -p "${subsystem_dir}"
  chown "${catj_user}" -R "${subsystem_dir}"

  if [ -d "${subsystem_dir}" ]; then
    echo "Register subsystem ${subsystem} for ${catj_user} ok"
  else
    echo "Register subsystem ${subsystem} for ${catj_user} fails"
    exit 1
  fi

  ls -l "/sys/fs/cgroup/${catj_user}/${subsystem}"
done
