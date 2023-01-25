#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

int main() {
  printf("uid %d\ngid %d\n", getuid(), getgid());
  return 0;
}
