using Celeste.Mod.Entities;
using Microsoft.Xna.Framework;

namespace Celeste.Mod.CelesteGymPlayground;

/// <summary>
/// Invisible map-owned training region. The stable TriggerId binds this
/// Celeste entity to a module or finish reference in the training script.
/// </summary>
[CustomEntity("CelesteGym/trainingTrigger")]
public sealed class TrainingTrigger : Trigger {
    public string TriggerId { get; }

    public TrainingTrigger(EntityData data, Vector2 offset)
        : base(data, offset) {
        TriggerId = data.Attr("triggerId");
    }
}
