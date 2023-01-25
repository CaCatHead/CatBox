#include <stdio.h>
#include <stdlib.h>

const char* ensure(const char* t) {
  if (t == NULL) {
    return "null";
  } else {
    return t;
  }
}

int main(int argc, char *argv[]) {
  printf("%s,%s\n", ensure(getenv("ONLINE_JUDGE")), ensure(getenv("test")));
  return 0;
}
