using System;

class Program {
    static long Factorial(long n) {
        if (n <= 1) return 1;
        return n * Factorial(n - 1);
    }

    static void Main(string[] args) {
        Console.WriteLine(Factorial(5));
    }
}
