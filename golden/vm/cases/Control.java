package br.rustydroid.vm;

/**
 * Casos de controle de fluxo (opcodes núcleo do M2): loops, branches,
 * packed-switch/sparse-switch (switch Java compilado para tableswitch/
 * lookupswitch e re-emitido pelo D8 como packed-switch/sparse-switch DEX),
 * comparações encadeadas e ternários.
 */
public final class Control {
    private Control() {}

    /** Fibonacci iterativo — loop com acumuladores. */
    public static int fib(int n) {
        int a = 0;
        int b = 1;
        for (int i = 0; i < n; i++) {
            int t = a + b;
            a = b;
            b = t;
        }
        return a;
    }

    /** Fatorial iterativo em long (overflow de long com n=21 é comportamento). */
    public static long fact(int n) {
        long acc = 1;
        for (int i = 2; i <= n; i++) {
            acc *= i;
        }
        return acc;
    }

    /** Passos de Collatz até 1. */
    public static int collatz(int n) {
        int steps = 0;
        while (n != 1) {
            if ((n & 1) == 0) {
                n = n / 2;
            } else {
                n = 3 * n + 1;
            }
            steps++;
        }
        return steps;
    }

    /** Soma 1..n. */
    public static int loopSum(int n) {
        int s = 0;
        for (int i = 1; i <= n; i++) {
            s += i;
        }
        return s;
    }

    /** GCD de Euclides (loop com rem). */
    public static int gcd(int a, int b) {
        while (b != 0) {
            int t = a % b;
            a = b;
            b = t;
        }
        return a;
    }

    /** Casos contíguos → tableswitch/packed-switch. */
    public static int switchPacked(int v) {
        switch (v) {
            case 1: return 10;
            case 2: return 20;
            case 3: return 30;
            case 4: return 40;
            case 5: return 50;
            default: return -100;
        }
    }

    /** Casos espalhados → lookupswitch/sparse-switch. */
    public static int switchSparse(int v) {
        switch (v) {
            case -77: return 1;
            case 1: return 2;
            case 1000: return 3;
            case 100000: return 4;
            default: return -1;
        }
    }

    /** Comparação encadeada estilo compareTo com condição composta. */
    public static int compareChain(int a, int b) {
        if (a < b) {
            return -1;
        }
        if (a > b) {
            return 1;
        }
        if (a == 0 && b == 0) {
            return 7;
        }
        return 0;
    }

    /** Ternário = max. */
    public static int ternary(int a, int b) {
        return a > b ? a : b;
    }

    /** Ternário aninhado (sinal). */
    public static int sign(int v) {
        return v > 0 ? 1 : (v < 0 ? -1 : 0);
    }
}
