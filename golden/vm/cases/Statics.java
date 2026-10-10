package br.rustydroid.vm;

/**
 * Casos de campos estáticos (opcodes núcleo do M2): sget/sput, inicialização
 * de classe via <clinit> (blocos static com contas) e via static_values
 * (ConstantValue do javac → encoded_array do DEX).
 *
 * Todos os métodos são determinísticos POR INVOCAÇÃO (o lado VM executa cada
 * caso num processo novo; o lado JVM roda tudo num processo só — então nenhum
 * caso pode depender do estado deixado por outro caso).
 */
public class Statics {
    /** Constante literal → ConstantValue → static_values no DEX. */
    public static final int ANSWER = 42;

    /** Inicializado no <clinit>. */
    public static int base = 100;

    /** Calculado no <clinit> (contas reais, não constantes dobradas). */
    public static int derived;

    /** Mutado dentro da chamada (determinístico por invocação). */
    public static int mut;

    static {
        derived = base * 3 + 7;
    }

    public static int answerValue() { return ANSWER; }

    public static int derivedValue() { return derived; }

    /** Lê dois statics (base e derived) + parâmetro. */
    public static int staticSum(int extra) { return base + derived + extra; }

    /** Escreve e lê um static dentro da mesma chamada. */
    public static int setAndRead(int v) {
        mut = v * 2;
        return mut;
    }

    /** <clinit> + campo + parâmetro (base * k + derived - base). */
    public static int clinitMath(int k) { return base * k + derived - base; }

    /** Caso void: muta static, não retorna nada (VM responde result null). */
    public static void touchVoid(int v) { mut = v; }
}
