package tests;

public class Assert {
	public static void yes(boolean value) {
		if (!value) {
			blowUp();
		}
	}
	public static native void blowUp();

}
