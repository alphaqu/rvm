public class ArrayTest {
	public static int[] singleArray(int value) {
		int[] ints = {
				value
		};
		return ints;
	}

	public static Object[] singleRefArray() {
		Object[] ints = {
				new Object(),
				null
		};
		return ints;
	}

	public static int[][] multiArray(int value) {
		int[][] ints = new int[4][4];
		return ints;
	}

	public static void setValue(int[] array, int index, int value) {
		array[index] = value;
	}

	public static int getValue(int[] array, int index) {
		return array[index];
	}

	public static void setValueRef(Object[] array, int index, Object value) {
		array[index] = value;
	}

	public static Object getValueRef(Object[] array, int index) {
		return array[index];
	}
}
