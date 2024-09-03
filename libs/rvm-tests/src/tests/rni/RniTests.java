package tests.rni;

public class RniTests {

	public static long test(int number1, long number2, int number3) {
		return testNative(number1, number2, number3);
	}


	public static native long testNative(int number1, long number2, int number3);
}
