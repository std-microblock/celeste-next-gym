using System.Reflection;
using Monocle;

namespace Celeste.Mod.CelesteGymCollector;

internal static class GymRandomPolicy {
    private static readonly FieldInfo RandomStackField = typeof(Calc).GetField(
        "randomStack",
        BindingFlags.NonPublic | BindingFlags.Static
    ) ?? throw new MissingFieldException(typeof(Calc).FullName, "randomStack");

    public static void Reset(int seed) {
        RandomStack.Clear();
        Calc.Random = new Random(seed);
    }

    private static Stack<Random> RandomStack =>
        RandomStackField.GetValue(null) as Stack<Random>
        ?? throw new InvalidOperationException("Monocle.Calc.randomStack is unavailable");
}
