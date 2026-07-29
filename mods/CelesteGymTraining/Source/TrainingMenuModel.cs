namespace Celeste.Mod.CelesteGymTraining;

public enum TrainingMenuFocus {
    Project,
    Start,
    Back
}

public sealed record TrainingProjectOption(string Id, string Title, string Summary, string Room);

public sealed class TrainingMenuModel {
    private readonly IReadOnlyList<TrainingProjectOption> projects;

    public TrainingMenuModel(IReadOnlyList<TrainingProjectOption> projects) {
        if (projects.Count == 0) throw new ArgumentException("at least one training project is required", nameof(projects));
        this.projects = projects;
    }

    public int ProjectIndex { get; private set; }
    public TrainingMenuFocus Focus { get; private set; } = TrainingMenuFocus.Project;
    public TrainingProjectOption SelectedProject => projects[ProjectIndex];
    public int ProjectCount => projects.Count;

    public TrainingProjectOption ProjectAt(int index) => projects[index];

    public void MoveUp() {
        switch (Focus) {
            case TrainingMenuFocus.Project when ProjectIndex > 0:
                ProjectIndex--;
                break;
            case TrainingMenuFocus.Start or TrainingMenuFocus.Back:
                Focus = TrainingMenuFocus.Project;
                break;
        }
    }

    public void MoveDown() {
        if (Focus == TrainingMenuFocus.Project && ProjectIndex < projects.Count - 1) {
            ProjectIndex++;
        } else if (Focus == TrainingMenuFocus.Project) {
            Focus = TrainingMenuFocus.Start;
        }
    }

    public void MoveLeft() {
        if (Focus == TrainingMenuFocus.Back) Focus = TrainingMenuFocus.Start;
    }

    public void MoveRight() {
        if (Focus == TrainingMenuFocus.Start) Focus = TrainingMenuFocus.Back;
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
