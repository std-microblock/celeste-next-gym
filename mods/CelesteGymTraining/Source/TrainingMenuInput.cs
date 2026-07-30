using Monocle;

namespace Celeste.Mod.CelesteGymTraining;

internal sealed class TrainingMenuInput {
    internal VirtualButton Left { get; } = Create(Settings.Instance.Left);
    internal VirtualButton Right { get; } = Create(Settings.Instance.Right);
    internal VirtualButton Up { get; } = Create(Settings.Instance.Up);
    internal VirtualButton Down { get; } = Create(Settings.Instance.Down);

    internal void Deregister() {
        Left.Deregister();
        Right.Deregister();
        Up.Deregister();
        Down.Deregister();
    }

    private static VirtualButton Create(Binding binding) {
        VirtualButton button = new(binding, Input.Gamepad, 0f, 0.4f);
        button.SetRepeat(0.4f, 0.1f);
        return button;
    }
}
