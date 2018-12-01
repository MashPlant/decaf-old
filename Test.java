class Test {
    int x;

    public static void main(String[] args) {
    }

    public void f1(int []a) {
        int t = a[0]++;
    }

    public void f2(int []a) {
        int t = ++a[0];
    }
    public void f3() {
        int t = x++;
    }
    public void f4() {
        int t = ++x;
    }
}