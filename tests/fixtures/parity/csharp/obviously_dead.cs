using System;

class Program {
    private static void HelperFunction() {
        Console.WriteLine("This is never called");
    }

    static void Main(string[] args) {
        Console.WriteLine("Entry point");
    }
}
