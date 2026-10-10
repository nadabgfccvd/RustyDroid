package br.rustydroid.vm;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;

/**
 * GoldenRunner — lado JVM real do harness golden do M2.
 *
 * Executa a tabela de casos via reflexão sobre {@code br.rustydroid.vm.*}
 * (métodos public static puros) e imprime UMA linha TSV por caso:
 *
 *   classe|metodo|sig|argsRepr|resultadoRepr
 *
 * - argsRepr: EXATAMENTE os literais da tabela (formato aceito por
 *   rd_vm::parse_arg_value: 42, -7, 1L, 1.5f, 2.5, 'c', "texto", true) —
 *   é a mesma string passada ao --args no lado da VM, garantindo que os dois
 *   lados recebam valores idênticos.
 * - resultadoRepr:
 *     primitivos      → String.valueOf estilo Java (char cru; float/double
 *                       via Float.toString/Double.toString)
 *     String          → conteúdo literal (os casos não usam '|' em strings)
 *     void            → <void>
 *     null            → null
 *     exceção escapada→ EXCEPTION:NomeClasse:mensagem (getMessage(); se null,
 *                       mensagem vazia)
 *
 * O run-golden.sh compara campo a campo com a resposta JSON do
 * `rd vm exec ... --json` (contrato 5-b) e normaliza boolean/char (a VM
 * devolve int cru no JSON; ver comparador).
 */
public final class GoldenRunner {
    private GoldenRunner() {}

    /** Tabela de casos: classe|metodo|sig|args (args = literais Java crus). */
    private static final String[] CASES = {
        // ── Arith: int ──────────────────────────────────────────────────────
        "Arith|addII|(II)I|42, 99",
        "Arith|addII|(II)I|-7, 3",
        "Arith|subII|(II)I|5, 100",
        "Arith|mulII|(II)I|-13, 21",
        "Arith|mulII|(II)I|65536, 65536",
        "Arith|divII|(II)I|-144, 12",
        "Arith|divII|(II)I|-2147483648, -1",
        "Arith|remII|(II)I|-17, 5",
        "Arith|remII|(II)I|-2147483648, -1",
        // ── Arith: long ─────────────────────────────────────────────────────
        "Arith|addJJ|(JJ)J|1000000000000L, -999999999999L",
        "Arith|subJJ|(JJ)J|-5000000000L, 2500000000L",
        "Arith|mulJJ|(JJ)J|3037000500L, 3L",
        "Arith|divJJ|(JJ)J|-1000000000000L, 7L",
        "Arith|remJJ|(JJ)J|1000000000007L, 1000L",
        // ── Arith: shifts (contagem via variável) ───────────────────────────
        "Arith|shlII|(II)I|1, 5",
        "Arith|shlII|(II)I|1, 33",
        "Arith|shrII|(II)I|-64, 4",
        "Arith|ushrII|(II)I|-64, 4",
        "Arith|shlJJ|(JI)J|1L, 40",
        "Arith|shrJJ|(JI)J|-1024L, 3",
        "Arith|ushrJJ|(JI)J|-1L, 1",
        // ── Arith: bitwise ──────────────────────────────────────────────────
        "Arith|andII|(II)I|61680, 4080",
        "Arith|orII|(II)I|61680, 4080",
        "Arith|xorII|(II)I|61680, 4080",
        "Arith|notII|(I)I|0",
        "Arith|andJJ|(JJ)J|1234567890123L, 987654321L",
        "Arith|orJJ|(JJ)J|1234567890123L, 987654321L",
        "Arith|xorJJ|(JJ)J|1234567890123L, 987654321L",
        "Arith|notJJ|(J)J|1234567890123L",
        "Arith|negII|(I)I|-2147483647",
        "Arith|negJJ|(J)J|-9000000000000000000L",
        // ── Arith: casts ────────────────────────────────────────────────────
        "Arith|i2b|(I)B|300",
        "Arith|i2b|(I)B|-129",
        "Arith|i2c|(I)C|97",
        "Arith|i2c|(I)C|65631",
        "Arith|i2s|(I)S|70000",
        "Arith|i2s|(I)S|-70001",
        "Arith|l2i|(J)I|4294967299L",
        "Arith|i2l|(I)J|-5",
        "Arith|i2f|(I)F|123456789",
        "Arith|i2d|(I)D|-2",
        "Arith|l2f|(J)F|1000000000000000000L",
        "Arith|l2d|(J)D|9007199254740993L",
        "Arith|f2d|(F)D|1.5f",
        "Arith|d2f|(D)F|0.1",
        "Arith|d2i|(D)I|3.99",
        "Arith|d2i|(D)I|-3.99",
        "Arith|d2l|(D)J|1.0e18",
        "Arith|d2l|(D)J|Infinity",
        "Arith|f2i|(F)I|-1.5f",
        "Arith|f2i|(F)I|NaNf",
        "Arith|f2l|(F)J|3.5e10f",
        // ── Arith: float/double ─────────────────────────────────────────────
        "Arith|addFF|(FF)F|0.1f, 0.2f",
        "Arith|mulFF|(FF)F|1.5f, -2.25f",
        "Arith|divFF|(FF)F|1.0f, 0.0f",
        "Arith|divFF|(FF)F|-1.0f, 0.0f",
        "Arith|divFF|(FF)F|0.0f, 0.0f",
        "Arith|remFF|(FF)F|5.5f, 2.0f",
        "Arith|addDD|(DD)D|0.1, 0.2",
        "Arith|mulDD|(DD)D|1.5, -2.5",
        "Arith|divDD|(DD)D|1.0, 3.0",
        "Arith|remDD|(DD)D|5.5, 2.0",
        "Arith|negDD|(D)D|-0.0",
        "Arith|ltDD|(DD)Z|0.5, 1.5",
        "Arith|ltDD|(DD)Z|NaN, 1.0",
        "Arith|geDD|(DD)Z|NaN, 1.0",
        "Arith|ltFF|(FF)Z|0.25f, 0.5f",
        "Arith|eqNanD|()Z|",
        "Arith|ltNanD|()Z|",
        "Arith|geNanD|()Z|",
        "Arith|ltNanF|()Z|",
        // ── Arith: boolean ──────────────────────────────────────────────────
        "Arith|boolAnd|(ZZ)Z|true, false",
        "Arith|boolOr|(ZZ)Z|false, false",
        "Arith|boolXor|(ZZ)Z|true, true",
        "Arith|boolNot|(Z)Z|true",
        // ── Arith: intrínsecos Math ─────────────────────────────────────────
        "Arith|sqrtD|(D)D|2.0",
        "Arith|minII|(II)I|-3, 7",
        "Arith|maxII|(II)I|-3, 7",
        "Arith|absII|(I)I|-42",
        // ── Control ─────────────────────────────────────────────────────────
        "Control|fib|(I)I|20",
        "Control|fib|(I)I|0",
        "Control|fact|(I)J|20",
        "Control|collatz|(I)I|27",
        "Control|loopSum|(I)I|100",
        "Control|gcd|(II)I|462, 1071",
        "Control|switchPacked|(I)I|3",
        "Control|switchPacked|(I)I|99",
        "Control|switchSparse|(I)I|1000",
        "Control|switchSparse|(I)I|-77",
        "Control|switchSparse|(I)I|12345",
        "Control|compareChain|(II)I|5, 3",
        "Control|compareChain|(II)I|3, 5",
        "Control|compareChain|(II)I|0, 0",
        "Control|ternary|(II)I|-10, 2",
        "Control|sign|(I)I|-5",
        "Control|sign|(I)I|0",
        // ── Arrays ──────────────────────────────────────────────────────────
        "Arrays|fillSum|(II)I|10, 3",
        "Arrays|fillSum|(II)I|1, 7",
        "Arrays|longArrSum|(I)J|5",
        "Arrays|arrReadWrite|(I)I|6",
        "Arrays|aioobe|(I)I|2",
        "Arrays|aioobe|(I)I|10",
        "Arrays|aioobe|(I)I|-1",
        "Arrays|charArrSum|(I)I|5",
        "Arrays|negArrayLen|()I|",
        // ── Strings ─────────────────────────────────────────────────────────
        "Strings|hashConst|()I|",
        "Strings|hashConstDigits|()I|",
        "Strings|hashEmpty|()I|",
        "Strings|hashArg|(Ljava/lang/String;)I|\"collatz\"",
        "Strings|lengthOf|(Ljava/lang/String;)I|\"RustyDroid\"",
        "Strings|isEmptyOf|(Ljava/lang/String;)Z|\"x\"",
        "Strings|isEmptyOf|(Ljava/lang/String;)Z|\"\"",
        "Strings|concatRet|(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;|\"Rusty\", \"Droid\"",
        "Strings|concatRet|(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;|\"\", \"x\"",
        "Strings|substringRet|(Ljava/lang/String;II)Ljava/lang/String;|\"RustyDroid\", 5, 10",
        "Strings|sbToString|(ILjava/lang/String;)Ljava/lang/String;|42, \"-ok\"",
        "Strings|concatLen|(Ljava/lang/String;Ljava/lang/String;)I|\"Rusty\", \"Droid\"",
        "Strings|concatHash|(Ljava/lang/String;Ljava/lang/String;)I|\"Rusty\", \"Droid\"",
        "Strings|concatEquals|(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Z|\"a\", \"b\", \"ab\"",
        "Strings|concatEquals|(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Z|\"a\", \"b\", \"ac\"",
        "Strings|charAtOk|(Ljava/lang/String;I)I|\"Rusty\", 0",
        "Strings|charAtOk|(Ljava/lang/String;I)I|\"Droid\", 4",
        "Strings|charAtOob|(Ljava/lang/String;I)I|\"Rusty\", 99",
        "Strings|charAtOob|(Ljava/lang/String;I)I|\"Rusty\", -1",
        "Strings|substringLen|(Ljava/lang/String;II)I|\"RustyDroid\", 5, 10",
        "Strings|substringHash|(Ljava/lang/String;II)I|\"RustyDroid\", 0, 5",
        "Strings|substringEquals|(Ljava/lang/String;IILjava/lang/String;)Z|\"RustyDroid\", 5, 10, \"Droid\"",
        "Strings|sbLen|(ILjava/lang/String;)I|42, \"-ok\"",
        "Strings|sbEqInt|(ILjava/lang/String;Ljava/lang/String;)Z|7, \"-ok\", \"7-ok\"",
        "Strings|sbHashLong|(J)I|1234567890123L",
        "Strings|valueOfLen|(I)I|123",
        "Strings|valueOfEqDouble|(D)Z|2.5",
        // ── Statics ─────────────────────────────────────────────────────────
        "Statics|answerValue|()I|",
        "Statics|derivedValue|()I|",
        "Statics|staticSum|(I)I|5",
        "Statics|setAndRead|(I)I|21",
        "Statics|clinitMath|(I)I|2",
        "Statics|touchVoid|(I)V|5",
        // ── Exceptions ──────────────────────────────────────────────────────
        "Exceptions|divZeroCaught|(I)I|4",
        "Exceptions|divZeroCaught|(I)I|0",
        "Exceptions|throwCaught|(I)I|7",
        "Exceptions|throwCaught|(I)I|-5",
        "Exceptions|nullStringLen|(Ljava/lang/String;)I|null",
        "Exceptions|customMsgLen|()I|",
        "Exceptions|customHash|()I|",
        "Exceptions|rethrow|()I|",
        "Exceptions|escapeDivZero|()I|",
        "Exceptions|escapeIAE|(I)I|5",
        "Exceptions|catchWide|(I)I|9",
        "Exceptions|catchWide|(I)I|1",
    };

    public static void main(String[] argv) {
        StringBuilder out = new StringBuilder(CASES.length * 64);
        for (String line : CASES) {
            String[] p = line.split("\\|", 4);
            String cls = p[0];
            String met = p[1];
            String sig = p[2];
            String argsRaw = p.length > 3 ? p[3] : "";
            Class<?> ret = retType(sig);
            Object result;
            try {
                Method m = resolve(cls, met, sig);
                result = m.invoke(null, boxArgs(sig, argsRaw));
            } catch (InvocationTargetException ite) {
                result = ite.getTargetException();
            } catch (Throwable t) {
                result = t; // falha do harness aparece como EXCEPTION na tabela
            }
            out.append(cls).append('|')
               .append(met).append('|')
               .append(sig).append('|')
               .append(argsRaw).append('|')
               .append(repr(result, ret)).append('\n');
        }
        System.out.print(out);
    }

    // ── descritores ─────────────────────────────────────────────────────────

    /** Tipo de retorno de (params)ret. */
    private static Class<?> retType(String sig) {
        String ret = sig.substring(sig.lastIndexOf(')') + 1);
        switch (ret) {
            case "V": return void.class;
            case "I": return int.class;
            case "J": return long.class;
            case "F": return float.class;
            case "D": return double.class;
            case "Z": return boolean.class;
            case "B": return byte.class;
            case "S": return short.class;
            case "C": return char.class;
            default: return String.class; // casos do harness só retornam String
        }
    }

    /** Quebra (params) em descritores. */
    private static String[] paramTypes(String sig) {
        String params = sig.substring(1, sig.lastIndexOf(')'));
        java.util.List<String> out = new java.util.ArrayList<>();
        int i = 0;
        while (i < params.length()) {
            int start = i;
            while (params.charAt(i) == '[') i++;
            if (params.charAt(i) == 'L') {
                while (params.charAt(i) != ';') i++;
            }
            i++;
            out.add(params.substring(start, i));
        }
        return out.toArray(new String[0]);
    }

    // ── resolução reflexiva ─────────────────────────────────────────────────

    private static Method resolve(String cls, String met, String sig) throws Exception {
        String[] descs = paramTypes(sig);
        Class<?>[] types = new Class<?>[descs.length];
        for (int i = 0; i < descs.length; i++) {
            types[i] = paramClass(descs[i]);
        }
        Class<?> c = Class.forName("br.rustydroid.vm." + cls);
        return c.getDeclaredMethod(met, types);
    }

    private static Class<?> paramClass(String desc) {
        switch (desc) {
            case "I": return int.class;
            case "J": return long.class;
            case "F": return float.class;
            case "D": return double.class;
            case "Z": return boolean.class;
            case "B": return byte.class;
            case "S": return short.class;
            case "C": return char.class;
            case "Ljava/lang/String;": return String.class;
            default: throw new IllegalArgumentException("param não suportado: " + desc);
        }
    }

    // ── parsing de literais Java (mesma gramática do --args da rd) ──────────

    /** Converte os literais crus (string --args) em Object[] boxado. */
    private static Object[] boxArgs(String sig, String argsRaw) {
        String[] descs = paramTypes(sig);
        if (descs.length == 0) {
            return new Object[0];
        }
        String[] lits = splitLiterals(argsRaw, descs.length);
        Object[] out = new Object[descs.length];
        for (int i = 0; i < descs.length; i++) {
            out[i] = boxOne(lits[i].trim(), descs[i]);
        }
        return out;
    }

    /** Split por vírgula (os casos do harness não usam vírgula em strings). */
    private static String[] splitLiterals(String raw, int n) {
        String[] out = new String[n];
        int idx = 0;
        int start = 0;
        for (int i = 0; i < raw.length() && idx < n - 1; i++) {
            if (raw.charAt(i) == ',') {
                out[idx++] = raw.substring(start, i);
                start = i + 1;
            }
        }
        out[idx] = raw.substring(start);
        return out;
    }

    private static Object boxOne(String lit, String desc) {
        switch (desc) {
            case "I": return Integer.parseInt(lit);
            case "B": return Byte.valueOf((byte) Integer.parseInt(lit));
            case "S": return Short.valueOf((short) Integer.parseInt(lit));
            case "Z": return Boolean.valueOf(lit.equals("true"));
            case "C": return Character.valueOf(unquoteChar(lit));
            case "J": return Long.parseLong(stripSuffix(lit, "L"));
            case "F": return Float.valueOf(parseFloatLit(stripSuffix(lit, "f")));
            case "D": return Double.valueOf(parseDoubleLit(stripSuffix(lit, "d")));
            case "Ljava/lang/String;":
                if (lit.equals("null")) return null;
                return unquoteString(lit);
            default: throw new IllegalArgumentException("literal p/ " + desc + "?");
        }
    }

    private static String stripSuffix(String s, String suf) {
        return s.endsWith(suf) ? s.substring(0, s.length() - 1) : s;
    }

    private static float parseFloatLit(String s) {
        String low = s.toLowerCase();
        if (low.equals("nan")) return Float.NaN;
        if (low.equals("infinity") || low.equals("inf")) return Float.POSITIVE_INFINITY;
        if (low.equals("-infinity") || low.equals("-inf")) return Float.NEGATIVE_INFINITY;
        return Float.parseFloat(s);
    }

    private static double parseDoubleLit(String s) {
        String low = s.toLowerCase();
        if (low.equals("nan")) return Double.NaN;
        if (low.equals("infinity") || low.equals("inf")) return Double.POSITIVE_INFINITY;
        if (low.equals("-infinity") || low.equals("-inf")) return Double.NEGATIVE_INFINITY;
        return Double.parseDouble(s);
    }

    /** 'a' / '\n' → char (escapes mínimos, espelhando parse_arg_value). */
    private static char unquoteChar(String lit) {
        if (lit.length() >= 3 && lit.startsWith("'") && lit.endsWith("'")) {
            String inner = lit.substring(1, lit.length() - 1);
            switch (inner) {
                case "\\n": return '\n';
                case "\\t": return '\t';
                case "\\\\": return '\\';
                case "\\'": return '\'';
                default:
                    if (inner.length() == 1) return inner.charAt(0);
                    throw new IllegalArgumentException("char literal ruim: " + lit);
            }
        }
        throw new IllegalArgumentException("char literal sem aspas: " + lit);
    }

    /** "txt" → txt com escapes mínimos (\n \t \\ \"). */
    private static String unquoteString(String lit) {
        if (lit.length() >= 2 && lit.startsWith("\"") && lit.endsWith("\"")) {
            String inner = lit.substring(1, lit.length() - 1);
            StringBuilder out = new StringBuilder(inner.length());
            for (int i = 0; i < inner.length(); i++) {
                char c = inner.charAt(i);
                if (c == '\\' && i + 1 < inner.length()) {
                    char n = inner.charAt(++i);
                    switch (n) {
                        case 'n': out.append('\n'); break;
                        case 't': out.append('\t'); break;
                        case '\\': out.append('\\'); break;
                        case '"': out.append('"'); break;
                        case '\'': out.append('\''); break;
                        default: out.append('\\').append(n); break;
                    }
                } else {
                    out.append(c);
                }
            }
            return out.toString();
        }
        return lit; // sem aspas: literal cru (compatível com parse_arg_value)
    }

    // ── representação do resultado (contrato do TSV) ────────────────────────

    private static String repr(Object r, Class<?> ret) {
        if (ret == void.class) return "<void>";
        if (r == null) return "null";
        if (r instanceof Throwable) {
            Throwable t = (Throwable) r;
            String msg = t.getMessage();
            return "EXCEPTION:" + t.getClass().getName() + ":" + (msg == null ? "" : msg);
        }
        if (r instanceof String) return (String) r;
        if (r instanceof Float) return Float.toString((Float) r);
        if (r instanceof Double) return Double.toString((Double) r);
        if (r instanceof Character) return String.valueOf((Character) r);
        if (r instanceof Boolean) return String.valueOf((Boolean) r);
        if (r instanceof Number) return String.valueOf(r);
        return "<objeto:" + r.getClass().getName() + ">";
    }
}
