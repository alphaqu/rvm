package core;

public class Assert {
	public native static void yes(boolean value);

	public native static void eq(int left, int right);

	public native static void eq(long left, long right);

	public native static void eq(float left, float right);

	public native static void eq(double left, double right);
}
