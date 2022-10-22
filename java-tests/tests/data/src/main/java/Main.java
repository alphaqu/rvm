public class Main {
    public int age;
    public boolean allowedToDrink;

    public Main(int age, boolean allowedToDrink) {
        this.age = age;
        this.allowedToDrink = allowedToDrink;
    }

    public static void main(String[] args) {
        new Main(69, true);
    }

    public static native void hi(int var0);
}
