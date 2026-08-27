using System;
using System.Collections.Generic;

class Program {
    static Dictionary<string, Action> handlers = new Dictionary<string, Action>();

    static void Register(string name, Action fn) {
        handlers[name] = fn;
    }

    static void Main(string[] args) {
        Register("test", () => Console.WriteLine("Handler"));
        handlers["test"]();
    }
}
