package br.rustydroid.vm;

/**
 * Casos de arrays (opcodes núcleo do M2): new-array, aget/aput em int/long/char,
 * array-length, e exceção ArrayIndexOutOfBoundsException capturada por
 * try/catch (o handler precisa ser achado pelo tries/handlers do CodeItem).
 */
public final class Arrays {
    private Arrays() {}

    /** Cria, preenche a[i] = (i+1)*step e soma. */
    public static int fillSum(int n, int step) {
        int[] a = new int[n];
        for (int i = 0; i < n; i++) {
            a[i] = (i + 1) * step;
        }
        int s = 0;
        for (int i = 0; i < n; i++) {
            s += a[i];
        }
        return s;
    }

    /** Array de long com valores largos (aput-wide/aget-wide). */
    public static long longArrSum(int n) {
        long[] a = new long[n];
        for (int i = 0; i < n; i++) {
            a[i] = (long) (i + 1) * 1000000000000L;
        }
        long s = 0;
        for (int i = 0; i < n; i++) {
            s += a[i];
        }
        return s;
    }

    /** Escrita e leitura com índices calculados. */
    public static int arrReadWrite(int n) {
        int[] a = new int[n];
        for (int i = 0; i < n; i++) {
            a[i] = i * i;
        }
        int last = a[n - 1];
        a[0] = last * 2;
        return a[0] + a[n - 1] * 3;
    }

    /** Acesso fora dos limites capturado → -1; dentro dos limites → o valor. */
    public static int aioobe(int idx) {
        int[] a = new int[5];
        try {
            a[idx] = 9;
            return a[idx];
        } catch (ArrayIndexOutOfBoundsException e) {
            return -1;
        }
    }

    /** Array de char com aput/aget char (elementos 0..65535 mapeados em int). */
    public static int charArrSum(int n) {
        char[] c = new char[n];
        for (int i = 0; i < n; i++) {
            c[i] = (char) ('a' + i);
        }
        int s = 0;
        for (int i = 0; i < n; i++) {
            s += c[i];
        }
        return s;
    }

    /** Tamanho negativo capturado → -2. */
    public static int negArrayLen() {
        try {
            int[] a = new int[-1];
            return a.length;
        } catch (NegativeArraySizeException e) {
            return -2;
        }
    }
}
