package tests.object;

import tests.Assert;

public class InterfaceTest implements Fruit {
	int hi;

	public static void hi() {
		InterfaceTest interfaceTest1 = new InterfaceTest();
		interfaceTest1.hi = 543;
		Fruit interfaceTest = interfaceTest1;

		Assert.yes(interfaceTest.hello() == 543);
	}

	@Override
	public int hello() {
		return hi;
	}
}
