package tests.control_flow;

public class Java {
	public static int pow(int base, int power) {
		int result = 1;

		while (power > 0) {
			if (power % 2 == 1) {
				result = result * base;
			}

			base = base * base;
			power >>= 1;
		}

		return result;
	}
}
