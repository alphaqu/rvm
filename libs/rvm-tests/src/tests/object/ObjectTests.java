package testing.object;

public class ObjectTests {
	public static SimpleObject createSimple() {
		return new SimpleObject();
	}

	public static SimpleObject createSimpleNumbered(int value) {
		return new SimpleObject(value);
	}

	public static int getSimpleField(SimpleObject object) {
		return object.value;
	}

	public static void setSimpleField(SimpleObject object, int value) {
		object.value = value;
	}

	public static int simpleInvocation(SimpleObject object) {
		return object.basic();
	}

	public static ExtendedObject createExtended() {
		return new ExtendedObject(400, 500);
	}

	public static SimpleObject casting(ExtendedObject extendedObject) {
		return extendedObject;
	}

	public static int interfaceCall(Animal animal) {
		return animal.age();
	}
}
