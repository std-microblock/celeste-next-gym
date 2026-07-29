using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Input;
using Monocle;

namespace Celeste.Mod.CelesteGymTraining;

[Tracked]
public sealed class TrainingProjectMenu : Entity {
    private const float UiWidth = 320f;
    private const float UiHeight = 180f;
    private const float CardX = 24f;
    private const float CardWidth = 272f;
    private const float CardTop = 58f;
    private const float CardHeight = 30f;
    private const float CardGap = 4f;
    private static readonly Color Panel = new(10, 15, 29, 242);
    private static readonly Color Card = new(29, 41, 65, 255);
    private static readonly Color Highlight = new(92, 205, 214, 255);
    private static readonly Color Primary = new(240, 93, 118, 255);

    private readonly Level level;
    private readonly TrainingMenuModel model;

    public TrainingProjectMenu(Level level, IReadOnlyList<TrainingProjectOption> projects) {
        this.level = level;
        model = new TrainingMenuModel(projects);
        Tag = Tags.HUD | Tags.PauseUpdate | Tags.FrozenUpdate | Tags.TransitionUpdate;
        Depth = int.MinValue;
    }

    public override void Added(Scene scene) {
        base.Added(scene);
        level.Paused = true;
    }

    public override void Removed(Scene scene) {
        level.Paused = false;
        base.Removed(scene);
    }

    public override void Update() {
        base.Update();
        bool up = Input.MenuUp.Pressed || MInput.Keyboard.Pressed(Keys.W) || MInput.Keyboard.Pressed(Keys.Up);
        bool down = Input.MenuDown.Pressed || MInput.Keyboard.Pressed(Keys.S) || MInput.Keyboard.Pressed(Keys.Down);
        bool left = Input.MenuLeft.Pressed || MInput.Keyboard.Pressed(Keys.A) || MInput.Keyboard.Pressed(Keys.Left);
        bool right = Input.MenuRight.Pressed || MInput.Keyboard.Pressed(Keys.D) || MInput.Keyboard.Pressed(Keys.Right);

        if (up) model.MoveUp();
        else if (down) model.MoveDown();
        else if (left) model.MoveLeft();
        else if (right) model.MoveRight();

        UpdateMouse();

        if (Input.MenuCancel.Pressed || MInput.Keyboard.Pressed(Keys.Escape)) {
            Back();
        } else if (Input.MenuConfirm.Pressed || MInput.Keyboard.Pressed(Keys.Enter) || MInput.Keyboard.Pressed(Keys.Space)) {
            ActivateFocused();
        }
    }

    private void UpdateMouse() {
        Vector2 mouse = MousePositionInUi();
        Point mousePoint = new((int) mouse.X, (int) mouse.Y);
        for (int index = 0; index < model.ProjectCount; index++) {
            if (ProjectBounds(index).Contains(mousePoint)) {
                model.FocusProject(index);
                if (MInput.Mouse.PressedLeftButton) Start();
                return;
            }
        }
        if (StartBounds.Contains(mousePoint)) {
            model.FocusButton(TrainingMenuFocus.Start);
            if (MInput.Mouse.PressedLeftButton) Start();
        } else if (BackBounds.Contains(mousePoint)) {
            model.FocusButton(TrainingMenuFocus.Back);
            if (MInput.Mouse.PressedLeftButton) Back();
        }
    }

    private static Vector2 MousePositionInUi() {
        Rectangle viewport = Engine.Viewport.Bounds;
        Vector2 mouse = MInput.Mouse.Position - viewport.Location.ToVector2();
        if (viewport.Width <= 0 || viewport.Height <= 0) return mouse;
        return new Vector2(mouse.X * UiWidth / viewport.Width, mouse.Y * UiHeight / viewport.Height);
    }

    private void ActivateFocused() {
        if (model.Focus == TrainingMenuFocus.Back) Back();
        else Start();
    }

    private void Start() {
        TrainingProjectOption project = model.SelectedProject;
        level.Paused = false;
        level.Session.SetFlag(CelesteGymTrainingModule.TrainingActiveFlag);
        level.Session.Level = project.Room;
        level.Session.RespawnPoint = null;
        Engine.Scene = new LevelLoader(level.Session) {
            PlayerIntroTypeOverride = Player.IntroTypes.None
        };
    }

    private void Back() {
        level.Paused = false;
        Engine.Scene = new LevelExit(LevelExit.Mode.GiveUp, level.Session, level.HiresSnow);
    }

    public override void Render() {
        Draw.Rect(0f, 0f, UiWidth, UiHeight, Panel);
        Draw.Rect(0f, 0f, UiWidth, 4f, Primary);
        ChineseText.Draw("CELeste 训练场", new Vector2(24f, 17f), Vector2.Zero, 0.22f, Color.White);
        ChineseText.Draw("选择训练项目", new Vector2(24f, 39f), Vector2.Zero, 0.14f, new Color(176, 194, 222));

        for (int index = 0; index < model.ProjectCount; index++) RenderProject(index);
        RenderButton(StartBounds, "开始训练", model.Focus == TrainingMenuFocus.Start, Primary);
        RenderButton(BackBounds, "返回", model.Focus == TrainingMenuFocus.Back, Card);
        ChineseText.Draw("WASD / 方向键选择   Enter 确认   Esc 返回", new Vector2(160f, 171f), new Vector2(0.5f, 1f), 0.105f, new Color(143, 161, 190));
    }

    private void RenderProject(int index) {
        Rectangle bounds = ProjectBounds(index);
        bool selected = model.ProjectIndex == index;
        bool focused = selected && model.Focus == TrainingMenuFocus.Project;
        Draw.Rect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? new Color(36, 73, 91) : Card);
        Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? Highlight : new Color(57, 72, 98));
        TrainingProjectOption project = model.ProjectAt(index);
        ChineseText.Draw(project.Title, new Vector2(bounds.X + 9f, bounds.Y + 7f), Vector2.Zero, 0.145f, Color.White);
        ChineseText.Draw(project.Summary, new Vector2(bounds.X + 9f, bounds.Y + 21f), Vector2.Zero, 0.09f, new Color(175, 193, 220));
    }

    private static void RenderButton(Rectangle bounds, string label, bool focused, Color color) {
        Draw.Rect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? color : color * 0.65f);
        Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? Color.White : new Color(82, 95, 119));
        ChineseText.Draw(label, bounds.Center.ToVector2(), new Vector2(0.5f, 0.5f), 0.13f, Color.White);
    }

    private static Rectangle ProjectBounds(int index) => new((int) CardX, (int) (CardTop + index * (CardHeight + CardGap)), (int) CardWidth, (int) CardHeight);
    private static Rectangle StartBounds => new(84, 137, 92, 22);
    private static Rectangle BackBounds => new(184, 137, 52, 22);
}
