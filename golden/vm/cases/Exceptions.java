package br.rustydroid.vm;

/**
 * Casos de exceções (opcodes núcleo do M2): try/catch com tipos do runtime
 * (ArithmeticException, NullPointerException, StringIndexOutOfBoundsException,
 * ArrayIndexOutOfBoundsException), throw de exceção do usuário (classe custom
 * estendendo RuntimeException) e rethrow.
 *
 * Boom é uma classe irmã NO MESMO arquivo (a unidade compila isolada; herança
 * só até RuntimeException, permitida pelo escopo do harness).
 */
class Boom extends RuntimeException {
    Boom(String msg) {
        super(msg);
    }
}

public final class Exceptions {
    private Exceptions() {}

    /** Divisão por zero capturada → -1; divisão normal → 100/d. */
    public static int divZeroCaught(int d) {
        try {
            return 100 / d;
        } catch (ArithmeticException e) {
            return -1;
        }
    }

    /** throw + catch de IllegalArgumentException → -2; sem throw → v+1. */
    public static int throwCaught(int v) {
        try {
            if (v < 0) {
                throw new IllegalArgumentException("neg");
            }
            return v + 1;
        } catch (IllegalArgumentException e) {
            return -2;
        }
    }

    /**
     * NPE por invoke em receiver null, capturada → -3.
     * O null chega via --args 'null' (parse_arg_value → Value::Null): o
     * literal null do javac vira `const/4 v0, 0` no bytecode, que a VM mínima
     * ainda não reinterpreta como referência (bug anotado no README/worklog).
     */
    public static int nullStringLen(String s) {
        try {
            return s.length();
        } catch (NullPointerException e) {
            return -3;
        }
    }

    /** Exceção custom materializada: getMessage().length() → 6. */
    public static int customMsgLen() {
        try {
            throw new Boom("abcdef");
        } catch (Boom e) {
            return e.getMessage().length();
        }
    }

    /** Conteúdo da mensagem validado por hashCode (mesmo algoritmo de String). */
    public static int customHash() {
        try {
            throw new Boom("golden-vector");
        } catch (Boom e) {
            return e.getMessage().hashCode();
        }
    }

    /** Rethrow de outra classe: escapa como IllegalStateException. */
    public static int rethrow() {
        try {
            throw new Boom("primeira");
        } catch (Boom e) {
            throw new IllegalStateException("segunda");
        }
    }

    /** ArithmeticException escapada (sem catch). */
    public static int escapeDivZero() {
        return 1 / 0;
    }

    /** IllegalArgumentException escapada (sem catch). */
    public static int escapeIAE(int v) {
        if (v >= 0) {
            throw new IllegalArgumentException("iae-const");
        }
        return v;
    }

    /** Exceção lançada por instrução (aget fora dos limites) + catch amplo. */
    public static int catchWide(int idx) {
        int[] a = new int[3];
        try {
            return a[idx];
        } catch (RuntimeException e) {
            return -9;
        }
    }
}
