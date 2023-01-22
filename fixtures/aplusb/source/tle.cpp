#include <iostream>

using namespace std;

int main() {
  int a, b;
  cin >> a >> b;
  long long sum = 0;
  for (int i = 1; i <= (int) 2e9; i++) {
    sum += a + b;
    sum %= 998244353;
  }
  cout << sum << '\n';
  return 0;
}
