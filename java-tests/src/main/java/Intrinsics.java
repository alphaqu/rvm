public class Intrinsics {

    public static void assertEquals(int a, int b) {
        if (a != b) {
            throw new RuntimeException("a != b, a = " + a + ", b = " + b);
        }
    }
}
