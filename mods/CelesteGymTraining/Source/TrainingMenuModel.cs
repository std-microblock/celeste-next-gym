namespace Celeste.Mod.CelesteGymTraining;

public enum TrainingMenuFocus {
    Project,
    Start,
    Back
}

public sealed record TrainingProjectOption(
    string Id,
    string Title,
    string Summary,
    string Room,
    string Difficulty = "入门",
    string? Thumbnail = null
);

public sealed class TrainingMenuModel {
    public const int Rows = 2;
    private readonly IReadOnlyList<TrainingProjectOption> projects;

    public TrainingMenuModel(IReadOnlyList<TrainingProjectOption> projects) {
        if (projects.Count == 0) throw new ArgumentException("at least one training project is required", nameof(projects));
        this.projects = projects;
    }

    public int ProjectIndex { get; private set; }
    public TrainingMenuFocus Focus { get; private set; } = TrainingMenuFocus.Project;
    public TrainingProjectOption SelectedProject => projects[ProjectIndex];
    public int ProjectCount => projects.Count;
    public int ProjectColumn => ProjectIndex / Rows;

    public TrainingProjectOption ProjectAt(int index) => projects[index];

    public void MoveUp() {
        if (Focus is TrainingMenuFocus.Start or TrainingMenuFocus.Back) {
            Focus = TrainingMenuFocus.Project;
        } else if (ProjectIndex % Rows == 1) {
            ProjectIndex--;
        }
    }

    public void MoveDown() {
        if (Focus != TrainingMenuFocus.Project) return;
        if (ProjectIndex % Rows == 0 && ProjectIndex + 1 < projects.Count) {
            ProjectIndex++;
        } else {
            Focus = TrainingMenuFocus.Start;
        }
    }

    public void MoveLeft() {
        if (Focus == TrainingMenuFocus.Project && ProjectIndex >= Rows) ProjectIndex -= Rows;
        else if (Focus == TrainingMenuFocus.Back) Focus = TrainingMenuFocus.Start;
    }

    public void MoveRight() {
        if (Focus == TrainingMenuFocus.Project && ProjectIndex + Rows < projects.Count) ProjectIndex += Rows;
        else if (Focus == TrainingMenuFocus.Start) Focus = TrainingMenuFocus.Back;
    }

    public void FocusProject(int index) {
        if (index < 0 || index >= projects.Count) throw new ArgumentOutOfRangeException(nameof(index));
        ProjectIndex = index;
        Focus = TrainingMenuFocus.Project;
    }

    public void FocusButton(TrainingMenuFocus focus) {
        if (focus is not (TrainingMenuFocus.Start or TrainingMenuFocus.Back)) {
            throw new ArgumentOutOfRangeException(nameof(focus));
        }
        Focus = focus;
    }
}
