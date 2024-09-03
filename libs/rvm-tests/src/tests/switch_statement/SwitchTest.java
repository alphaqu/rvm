package tests.switch_statement;

import core.Assert;

public class SwitchTest {

	public static void test() {
		Assert.yes(testSwitch(0) == 420);
		Assert.yes(testSwitch(1) == 3);
		Assert.yes(testSwitch(2) == 2);
		Assert.yes(testSwitch(3) == 420);
		Assert.yes(testSwitch(4) == 4);
		Assert.yes(testSwitch(5) == 420);
		Assert.yes(testSwitch(10) == 10);
	}

	public static int testSwitch(int i) {
		int output = 0;
		switch (i) {
			case 1:
				output += 1;
			case 2:
				output += 2;
				break;

			case 4:
				output += 4;
				break;

			case 10:
				return 10;

			default:
				return 420;
		}

		return output;
	}
}