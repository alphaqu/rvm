public class Intrinsics {

    public static void assertEquals(int a, int b) {
        if (a != b) {
            // throw new RuntimeException("a != b, a = " + a + ", b = " + b);
            // Use concat instead of + to avoid InvokeDynamic for StringConcatFactory
            // Triggers CNFE for RuntimeException on rvm for now anyway
            throw new RuntimeException(
                    "a != b, a = ".concat(Integer.toString(a)).concat(", b = ").concat(Integer.toString(b)));
        }
    }
}
