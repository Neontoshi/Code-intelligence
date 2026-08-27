import java.util.HashMap;
import java.util.Map;

public class Main {
    interface Handler {
        void handle();
    }

    static Map<String, Handler> handlers = new HashMap<>();

    static void register(String name, Handler handler) {
        handlers.put(name, handler);
    }

    public static void main(String[] args) {
        register("test", () -> System.out.println("Handler"));
        handlers.get("test").handle();
    }
}
