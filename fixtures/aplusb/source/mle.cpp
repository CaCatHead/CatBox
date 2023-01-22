#include <iostream>

using namespace std;

const int mod = 998244353;
const int maxn = 100000000 + 5;

int n, a[maxn];

int main() {
  cin >> n >> a[1];
  for (int i = 2; i <= 100000000; i++) a[i] = a[i - 1] * 2 % mod;
  cout << a[maxn - 1] << '\n';
  return 0;
}
