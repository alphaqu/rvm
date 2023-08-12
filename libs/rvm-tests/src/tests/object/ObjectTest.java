public class ObjectTest {
	public int value;

	public ObjectTest(int value) {
		this.value = value;
	}

	public static Object newTest() {
		return new Object();
	}

	public static int simpleTest(int value) {
		return ObjectTest.simpleTestObject(value);
	}

	public static int simpleTestObject(int value) {
	    ObjectTest object =  new ObjectTest(value);
		return object.value;
	}
}
