package br.rustydroid.vm;

/**
 * Casos aritméticos puros (opcodes núcleo do M2): add/sub/mul/div/rem int e
 * long, shifts (com contagem via variável), bitwise, unários, casts numéricos,
 * aritmética float/double (incl. Infinity/NaN) e comparações que exercitam
 * cmpl/cmpg via operadores < e >=.
 *
 * Regras do harness: só métodos public static, sem autoboxing, sem concatenação
 * de strings com + (invokedynamic makeConcatWithConstants — fora do M2) e sem
 * recursos fora do escopo do interpretador mínimo.
 */
public final class Arith {
    private Arith() {}

    // ── int ─────────────────────────────────────────────────────────────────
    public static int addII(int a, int b) { return a + b; }
    public static int subII(int a, int b) { return a - b; }
    public static int mulII(int a, int b) { return a * b; }
    public static int divII(int a, int b) { return a / b; }
    public static int remII(int a, int b) { return a % b; }

    // ── long ────────────────────────────────────────────────────────────────
    public static long addJJ(long a, long b) { return a + b; }
    public static long subJJ(long a, long b) { return a - b; }
    public static long mulJJ(long a, long b) { return a * b; }
    public static long divJJ(long a, long b) { return a / b; }
    public static long remJJ(long a, long b) { return a % b; }

    // ── shifts: contagem via variável (força shift por registrador) ─────────
    public static int shlII(int v, int n) { int s = n; return v << s; }
    public static int shrII(int v, int n) { int s = n; return v >> s; }
    public static int ushrII(int v, int n) { int s = n; return v >>> s; }
    public static long shlJJ(long v, int n) { int s = n; return v << s; }
    public static long shrJJ(long v, int n) { int s = n; return v >> s; }
    public static long ushrJJ(long v, int n) { int s = n; return v >>> s; }

    // ── bitwise ─────────────────────────────────────────────────────────────
    public static int andII(int a, int b) { return a & b; }
    public static int orII(int a, int b) { return a | b; }
    public static int xorII(int a, int b) { return a ^ b; }
    public static int notII(int a) { return ~a; }
    public static long andJJ(long a, long b) { return a & b; }
    public static long orJJ(long a, long b) { return a | b; }
    public static long xorJJ(long a, long b) { return a ^ b; }
    public static long notJJ(long a) { return ~a; }

    // ── unários ─────────────────────────────────────────────────────────────
    public static int negII(int a) { return -a; }
    public static long negJJ(long a) { return -a; }

    // ── casts numéricos ─────────────────────────────────────────────────────
    public static byte i2b(int v) { return (byte) v; }
    public static char i2c(int v) { return (char) v; }
    public static short i2s(int v) { return (short) v; }
    public static int l2i(long v) { return (int) v; }
    public static long i2l(int v) { return (long) v; }
    public static float i2f(int v) { return (float) v; }
    public static double i2d(int v) { return (double) v; }
    public static float l2f(long v) { return (float) v; }
    public static double l2d(long v) { return (double) v; }
    public static double f2d(float v) { return (double) v; }
    public static float d2f(double v) { return (float) v; }
    public static int d2i(double v) { return (int) v; }
    public static long d2l(double v) { return (long) v; }
    public static int f2i(float v) { return (int) v; }
    public static long f2l(float v) { return (long) v; }

    // ── float / double (div por zero → Infinity; 0/0 → NaN) ─────────────────
    public static float addFF(float a, float b) { return a + b; }
    public static float mulFF(float a, float b) { return a * b; }
    public static float divFF(float a, float b) { return a / b; }
    public static float remFF(float a, float b) { return a % b; }
    public static double addDD(double a, double b) { return a + b; }
    public static double mulDD(double a, double b) { return a * b; }
    public static double divDD(double a, double b) { return a / b; }
    public static double remDD(double a, double b) { return a % b; }
    public static double negDD(double a) { return -a; }

    // comparações com NaN exercitam cmpl/cmpg/cmpg-float (semânticas distintas)
    public static boolean ltDD(double a, double b) { return a < b; }
    public static boolean geDD(double a, double b) { return a >= b; }
    public static boolean ltFF(float a, float b) { return a < b; }
    public static boolean eqNanD() { double nan = 0.0 / 0.0; return nan == nan; }
    public static boolean ltNanD() { double nan = 0.0 / 0.0; return nan < 1.0; }
    public static boolean geNanD() { double nan = 0.0 / 0.0; return nan >= 1.0; }
    public static boolean ltNanF() { float nan = 0.0f / 0.0f; return nan < 1.0f; }

    // ── boolean (iand/ior/ixor sobre 0/1) ───────────────────────────────────
    public static boolean boolAnd(boolean a, boolean b) { return a & b; }
    public static boolean boolOr(boolean a, boolean b) { return a | b; }
    public static boolean boolXor(boolean a, boolean b) { return a ^ b; }
    public static boolean boolNot(boolean a) { return !a; }

    // ── intrínsecos de plataforma exercitados via invoke-static ─────────────
    public static double sqrtD(double v) { return Math.sqrt(v); }
    public static int minII(int a, int b) { return Math.min(a, b); }
    public static int maxII(int a, int b) { return Math.max(a, b); }
    public static int absII(int a) { return Math.abs(a); }
}
