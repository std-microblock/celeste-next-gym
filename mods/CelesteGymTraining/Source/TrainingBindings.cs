using Microsoft.Xna.Framework.Input;
using Monocle;

namespace Celeste.Mod.CelesteGymTraining;

public static class TrainingBindings {
    private static bool applied;

    public static void ApplyDefaultsIfVanilla() {
        if (applied || Settings.Instance is null) return;
        applied = true;
        Settings settings = Settings.Instance;
        bool vanillaMovement = settings.Left.Keyboard.Contains(Keys.Left)
            && settings.Right.Keyboard.Contains(Keys.Right)
            && settings.Up.Keyboard.Contains(Keys.Up)
            && settings.Down.Keyboard.Contains(Keys.Down)
            && !settings.Left.Keyboard.Contains(Keys.A)
            && !settings.Right.Keyboard.Contains(Keys.D);
        if (!vanillaMovement) return;
        SetKeyboard(settings.Left, Keys.A);
        SetKeyboard(settings.Right, Keys.D);
        SetKeyboard(settings.Up, Keys.W);
        SetKeyboard(settings.Down, Keys.S);
        SetKeyboard(settings.Jump, Keys.L);
        SetKeyboard(settings.Dash, Keys.OemSemicolon);
        SetKeyboard(settings.Grab, Keys.OemQuotes);
    }

    private static void SetKeyboard(Binding binding, Keys key) {
        binding.ClearKeyboard();
        binding.Add(key);
    }
}
