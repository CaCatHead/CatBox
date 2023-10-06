#!/usr/bin/env optc

/// <reference path="/root/.optc/globals.d.ts" />

export default async function(version: string) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    return;
  }
  
  const toml = readTextFile('Cargo.toml');
  writeTextFile('Cargo.toml', toml.replace(/version = "\d+\.\d+\.\d+"/, `version = "${version}"`));
  writeTextFile('README.md', readTextFile('README.md').replace(/catj \d+\.\d+\.\d+/, `catj ${version}`));

  await $`git add .`;
  await $`git commit -m "chore: release v${version}"`;
  await $`git tag -a v${version} -m "chore: release v${version}"`;
  await $`git push --tags origin main`;
  await $`cargo publish`;
}
