package tests.object;

public class ExtendedObject extends SimpleObject implements Animal {
	public long anotherField;

	public ExtendedObject(long anotherField) {
		this.anotherField = anotherField;
	}

	public ExtendedObject(int value, long anotherField) {
		super(value);
		this.anotherField = anotherField;
	}

	@Override
	public int basic() {
		return super.basic() + 400;
	}

	@Override
	public int age() {
		return 49;
	}
}
