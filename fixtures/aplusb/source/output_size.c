#include <stdio.h>
#include <string.h>

int main(int argc, char *argv[]) {
  for (int i = 0; i < 5000000; i++) {
    printf("%s", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
  }
  return 0;
}
