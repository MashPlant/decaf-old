public class Test {
    public static void main(String[] args) {
    }
    public void f(int a) {
        boolean b = a == a;
        if (b) {
            a++;
        }else {
            a--;
        }
        if (!b) {
            a++;
        }else {
            a--;
        }
    }
}