package tests.statics;

public class Java {
	public static int number;

	static {
		number = 3;
	}

	public static void setStatic(int value) {
		number = value;
	}

	public static int getStatic() {
		return number;
	}
}
