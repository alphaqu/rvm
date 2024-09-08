package tests.exception;

public class Java {

	public static void basic(boolean shouldThrow) throws Exception {
		if (shouldThrow) {
			throw new Exception("Hello");
		}
	}
}
