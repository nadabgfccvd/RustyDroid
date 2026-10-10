package br.rustydroid.vm;

/**
 * Casos de strings (opcodes núcleo do M2 + intrínsecos de java.lang.String e
 * java.lang.StringBuilder): const-string, hashCode (algoritmo da spec sobre
 * unidades UTF-16), length, charAt, concat, substring e StringBuilder.
 *
 * IMPORTANTE (contrato do harness): nenhum método desta tabela RETORNA String.
 * O CLI `rd vm exec` hoje renderiza retorno de objeto como "<object>" (o
 * conteúdo fica no heap da VM, invisível ao JSON), então o conteúdo de strings
 * é validado INDIRETAMENTE mas com força: length + hashCode + equals sobre o
 * resultado — o bytecode ainda executa concat/substring/toString da mesma
 * forma; só o valor final do vetor é primitivo. Ver golden/vm/README.md.
 *
 * Sem concatenação com + (javac 21 geraria invokedynamic
 * makeConcatWithConstants, fora do M2) — usamos String.concat e
 * StringBuilder explícitos.
 */
public final class Strings {
    private Strings() {}

    // ── hashCode sobre constantes (algoritmo da spec) ───────────────────────
    public static int hashConst() { return "RustyDroid".hashCode(); }
    public static int hashConstDigits() { return "96354".hashCode(); }
    public static int hashEmpty() { return "".hashCode(); }
    public static int hashArg(String s) { return s.hashCode(); }

    // ── length / isEmpty ────────────────────────────────────────────────────
    public static int lengthOf(String s) { return s.length(); }
    public static boolean isEmptyOf(String s) { return s.isEmpty(); }

    // ── concat (conteúdo validado por length/hashCode/equals) ───────────────
    public static String concatRet(String a, String b) { return a.concat(b); }

    public static String substringRet(String s, int a, int b) { return s.substring(a, b); }

    public static String sbToString(int n, String suffix) {
        StringBuilder sb = new StringBuilder();
        sb.append(n);
        sb.append(suffix);
        return sb.toString();
    }

    public static int concatLen(String a, String b) { return a.concat(b).length(); }
    public static int concatHash(String a, String b) { return a.concat(b).hashCode(); }
    public static boolean concatEquals(String a, String b, String expected) {
        return a.concat(b).equals(expected);
    }

    // ── charAt ──────────────────────────────────────────────────────────────
    public static int charAtOk(String s, int i) { return s.charAt(i); }
    public static int charAtOob(String s, int i) {
        try {
            return s.charAt(i);
        } catch (StringIndexOutOfBoundsException e) {
            return -1;
        }
    }

    // ── substring ───────────────────────────────────────────────────────────
    public static int substringLen(String s, int a, int b) {
        return s.substring(a, b).length();
    }
    public static int substringHash(String s, int a, int b) {
        return s.substring(a, b).hashCode();
    }
    public static boolean substringEquals(String s, int a, int b, String expected) {
        return s.substring(a, b).equals(expected);
    }

    // ── StringBuilder (append + toString, conteúdo validado por equals) ─────
    public static int sbLen(int v, String tail) {
        StringBuilder sb = new StringBuilder();
        sb.append(v);
        sb.append(tail);
        return sb.length();
    }
    public static boolean sbEqInt(int v, String tail, String expected) {
        StringBuilder sb = new StringBuilder();
        sb.append(v).append(tail);
        return sb.toString().equals(expected);
    }
    public static int sbHashLong(long v) {
        StringBuilder sb = new StringBuilder();
        sb.append(v).append('!').append(true);
        return sb.toString().hashCode();
    }

    // ── String.valueOf (intrínseco static) ──────────────────────────────────
    public static int valueOfLen(int v) { return String.valueOf(v).length(); }
    public static boolean valueOfEqDouble(double v) {
        return String.valueOf(v).equals("2.5");
    }
}
