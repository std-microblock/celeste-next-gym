using Celeste.Mod.Entities;
using Microsoft.Xna.Framework;

namespace Celeste.Mod.CelesteGymTraining;

[CustomEntity("CelesteGymTraining/lessonTrigger")]
public sealed class TrainingLessonTrigger : Trigger {
    public string LessonId { get; }
    public string ProjectId { get; }

    public TrainingLessonTrigger(EntityData data, Vector2 offset) : base(data, offset) {
        LessonId = data.Attr("lessonId");
        ProjectId = data.Attr("projectId");
    }

    public override void OnEnter(Player player) {
        base.OnEnter(player);
        Controller?.SetNearbyLesson(LessonId, true);
    }

    public override void OnStay(Player player) {
        base.OnStay(player);
        Controller?.SetNearbyLesson(LessonId, true);
    }

    public override void OnLeave(Player player) {
        Controller?.SetNearbyLesson(LessonId, false);
        base.OnLeave(player);
    }

    private TrainingRuntimeController? Controller => Scene?.Tracker.GetEntity<TrainingRuntimeController>();
}

[CustomEntity("CelesteGymTraining/lessonEndTrigger")]
public sealed class TrainingLessonEndTrigger : Trigger {
    private readonly string lessonId;

    public TrainingLessonEndTrigger(EntityData data, Vector2 offset) : base(data, offset) {
        lessonId = data.Attr("lessonId");
    }

    public override void OnEnter(Player player) {
        base.OnEnter(player);
        Scene?.Tracker.GetEntity<TrainingRuntimeController>()?.ReachLessonEnd(lessonId);
    }
}

[CustomEntity("CelesteGymTraining/finishTrigger")]
public sealed class TrainingFinishTrigger : Trigger {
    private readonly string projectId;
    private readonly bool requireAllModules;

    public TrainingFinishTrigger(EntityData data, Vector2 offset) : base(data, offset) {
        projectId = data.Attr("projectId");
        requireAllModules = data.Bool("requireAllModules", true);
    }

    public override void OnEnter(Player player) {
        base.OnEnter(player);
        Scene?.Tracker.GetEntity<TrainingRuntimeController>()
            ?.ReachFinish(projectId, requireAllModules);
    }
}
