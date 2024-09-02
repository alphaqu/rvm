package tests.jni;

public class Main {
	static {
		System.loadLibrary("native");
	}

	public static void main() {
		new Main().sayHello();
	}

	private native void sayHello();
}