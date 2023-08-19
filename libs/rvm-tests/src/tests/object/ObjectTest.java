package tests.object;

public class ObjectTest {
	public int value;
	public ObjectTest child;

	public ObjectTest(int value, ObjectTest child) {
		this.value = value;
		this.child = child;
	}

	public int value() {
		return value;
	}

	public static Object newTest() {
		return new Object();
	}

	public static int simpleTest(int value) {
		return ObjectTest.simpleTestObject(value);
	}

	public static int simpleTestObject(int value) {
		ObjectTest object = new ObjectTest(value, null);
		return object.value;
	}

	public static int gcTest(int value) {
		for (int i = 0; i < 2; i++) {
			if (i == 1) {
				return value;
			}
		}

		ObjectTest objectTest1 = new ObjectTest(value, null);

		for (int i = 0; i < value; i = i + 1) {
			ObjectTest objectTest = new ObjectTest(value, null);
			for (int j = 0; j < 4; j = j + 1) {
				objectTest = new ObjectTest(value, objectTest);
			}
		}
		return objectTest1.value;
	}
}
