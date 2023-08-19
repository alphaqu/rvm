package tests.object;

import tests.Assert;

public class ExtendTest extends ObjectTest {
	public int another;

	public ExtendTest(int another, int value, ObjectTest child) {
		super(value, child);
		this.another = another;
	}


	public static void create() {
		ExtendTest test = new ExtendTest(69, 420, null);
		Assert.yes(test.value() == 420);
		Assert.yes(test.value == 420);
		Assert.yes(test.another == 69);
	}
}
