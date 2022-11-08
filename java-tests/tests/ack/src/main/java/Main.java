public class Main {

    public Main() {
    }

    static int ack(int m, int n) {
        if (m == 0) {
            return n + 1;
        } else if (m > 0 && n == 0) {
            return ack(m - 1, 1);
        } else if (m > 0 && n > 0) {
            return ack(m - 1, ack(m, n - 1));
        } else {
            return n + 1;
        }
    }

    public static int test() {
        return ack(3, 12);
    }

    public static boolean testZeroEq(int v) {
        return v == 0;
    }

    public static boolean testZeroNeq(int v) {
        return v != 0;
    }

    public static boolean testZeroGt(int v) {
        return v > 0;
    }

    public static boolean testZeroGe(int v) {
        return v >= 0;
    }

    public static boolean testZeroLt(int v) {
        return v < 0;
    }

    public static boolean testZeroLe(int v) {
        return v <= 0;
    }
}
