namespace Celeste.Mod.CelesteGymTraining.Test;

using Xunit;

public sealed class TrainingMenuModelTests {
    private static TrainingMenuModel Create() => new([
        new("one", "一", "", "one"),
        new("two", "二", "", "two"),
        new("three", "三", "", "three"),
        new("four", "四", "", "four"),
        new("five", "五", "", "five"),
        new("six", "六", "", "six")
    ]);

    [Fact]
    public void WasdStyleNavigationCoversProjectsAndButtons() {
        TrainingMenuModel model = Create();

        model.MoveDown();
        Assert.Equal(1, model.ProjectIndex);
        model.MoveDown();
        Assert.Equal(TrainingMenuFocus.Start, model.Focus);
        model.MoveRight();
        Assert.Equal(TrainingMenuFocus.Back, model.Focus);
        model.MoveLeft();
        Assert.Equal(TrainingMenuFocus.Start, model.Focus);
        model.MoveUp();
        Assert.Equal(TrainingMenuFocus.Project, model.Focus);
        Assert.Equal(1, model.ProjectIndex);
    }

    [Fact]
    public void HorizontalNavigationMovesBetweenMapCards() {
        TrainingMenuModel model = Create();

        model.MoveRight();
        Assert.Equal(2, model.ProjectIndex);
        model.MoveDown();
        Assert.Equal(3, model.ProjectIndex);
        model.MoveLeft();
        Assert.Equal(1, model.ProjectIndex);
        model.MoveUp();
        Assert.Equal(0, model.ProjectIndex);
    }

    [Fact]
    public void MouseStyleFocusSelectsEveryInteractiveTarget() {
        TrainingMenuModel model = Create();

        model.FocusProject(1);
        Assert.Equal("two", model.SelectedProject.Id);
        model.FocusButton(TrainingMenuFocus.Start);
        Assert.Equal(TrainingMenuFocus.Start, model.Focus);
        model.FocusButton(TrainingMenuFocus.Back);
        Assert.Equal(TrainingMenuFocus.Back, model.Focus);
    }
}
