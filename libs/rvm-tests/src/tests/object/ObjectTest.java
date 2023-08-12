public class ObjectTest {
	public int value;

	public ObjectTest(int value) {
		this.value = value;
	}

	public static Object newTest() {
		return new Object();
	}

	public static int simpleTest(int value) {
		ObjectTest objectTest = new ObjectTest(value);
		return objectTest.value;
	}

	public static ObjectTest simpleTestObject(int value) {
		return new ObjectTest(value);
	}
}
