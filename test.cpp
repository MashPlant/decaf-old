#include <cstring>

int *f(int *a, int n) {
  return a + n;
}

int main() {
  int a[10];
  memset(a, 0, sizeof(a));
}