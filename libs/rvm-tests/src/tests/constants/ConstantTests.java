package tests.constants;

import core.Assert;

public class ConstantTests {

	public static void test() {
		Assert.eq(iconst_m1(), -1);
		Assert.eq(iconst_0(), 0);
		Assert.eq(iconst_1(), 1);
		Assert.eq(iconst_2(), 2);
		Assert.eq(iconst_3(), 3);
		Assert.eq(iconst_4(), 4);
		Assert.eq(iconst_5(), 5);
		Assert.eq(lconst_0(), 0);
		Assert.eq(lconst_1(), 1);
		Assert.eq(fconst_0(), 0);
		Assert.eq(fconst_1(), 1);
		Assert.eq(fconst_2(), 2);
		Assert.eq(dconst_0(), 0);
		Assert.eq(dconst_1(), 1);
		Assert.eq(bipush(), 12);
		Assert.eq(sipush(), 244);
		Assert.eq(ldc(), 696969);
		Assert.eq(ldc_2(), 6969695232535242342L);
	}

	public static int iconst_m1() {
		return -1;
	}

	public static int iconst_0() {
		return 0;
	}

	public static int iconst_1() {
		return 1;
	}

	public static int iconst_2() {
		return 2;
	}

	public static int iconst_3() {
		return 3;
	}

	public static int iconst_4() {
		return 4;
	}

	public static int iconst_5() {
		return 5;
	}

	public static long lconst_0() {
		return 0;
	}

	public static long lconst_1() {
		return 1;
	}

	public static float fconst_0() {
		return 0;
	}

	public static float fconst_1() {
		return 1;
	}

	public static float fconst_2() {
		return 2;
	}

	public static double dconst_0() {
		return 0;
	}

	public static double dconst_1() {
		return 1;
	}

	public static int bipush() {
		return 12;
	}

	public static int sipush() {
		return 244;
	}

	public static int ldc() {
		return 696969;
	}

	public static long ldc_2() {
		return 6969695232535242342L;
	}
}
