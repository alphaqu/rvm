package testing.math;

import tests.Assert;

public class MathTests {
	public static int add(int left, int right) {return left + right;}
	public static long add(long left, long right) {return left + right;}
	public static float add(float left, float right) {return left + right;}
	public static double add(double left, double right) {return left + right;}

	public static int sub(int left, int right) {return left - right;}
	public static long sub(long left, long right) {return left - right;}
	public static float sub(float left, float right) {return left - right;}
	public static double sub(double left, double right) {return left - right;}

	public static int mul(int left, int right) {return left * right;}
	public static long mul(long left, long right) {return left * right;}
	public static float mul(float left, float right) {return left * right;}
	public static double mul(double left, double right) {return left * right;}

	public static int div(int left, int right) {return left / right;}
	public static long div(long left, long right) {return left / right;}
	public static float div(float left, float right) {return left / right;}
	public static double div(double left, double right) {return left / right;}

	public static int rem(int left, int right) {return left % right;}
	public static long rem(long left, long right) {return left % right;}
	public static float rem(float left, float right) {return left % right;}
	public static double rem(double left, double right) {return left % right;}

	public static int neg(int value) {return -value;}
	public static long neg(long value) {return -value;}
	public static float neg(float value) {return -value;}
	public static double neg(double value) {return -value;}

	public static int shl(int left, int right) {return left << right;}
	public static long shl(long left, long right) {return left << right;}

	public static int shr(int left, int right) {return left >> right;}
	public static long shr(long left, long right) {return left >> right;}

	public static int ushr(int left, int right) {return left >>> right;}
	public static long ushr(long left, long right) {return left >>> right;}

	public static int and(int left, int right) {return left & right;}
	public static long and(long left, long right) {return left & right;}

	public static int or(int left, int right) {return left | right;}
	public static long or(long left, long right) {return left | right;}

	public static int xor(int left, int right) {return left ^ right;}
	public static long xor(long left, long right) {return left ^ right;}
}
