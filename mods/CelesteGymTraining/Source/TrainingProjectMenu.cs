using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Input;
using Monocle;

namespace Celeste.Mod.CelesteGymTraining;

[Tracked]
public sealed class TrainingProjectMenu : Entity {
    private const float UiWidth = 1920f;
    private const float UiHeight = 1080f;
    private const int CardX = 120;
    private const int CardWidth = 390;
    private const int CardTop = 232;
    private const int CardHeight = 286;
    private const int CardColumnGap = 32;
    private const int CardRowGap = 26;
    private const int VisibleColumns = 4;
    private static readonly Color Overlay = new(5, 9, 19, 198);
    private static readonly Color Card = new(29, 41, 65, 255);
    private static readonly Color Highlight = new(92, 205, 214, 255);
    private static readonly Color Primary = new(240, 93, 118, 255);

    private readonly Level level;
    private readonly TrainingMenuModel model;
    private int firstVisibleColumn;

    public TrainingProjectMenu(Level level, IReadOnlyList<TrainingProjectOption> projects) {
        this.level = level;
        model = new TrainingMenuModel(projects);
        Tag = Tags.HUD | Tags.PauseUpdate | Tags.FrozenUpdate | Tags.TransitionUpdate;
        Depth = int.MinValue;
    }

    public override void Added(Scene scene) {
        base.Added(scene);
        ChineseText.Prepare();
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

        if (up || down || left || right) EnsureFocusedVisible();

        if (MInput.Mouse.WheelDelta > 0) ScrollColumns(-1);
        else if (MInput.Mouse.WheelDelta < 0) ScrollColumns(1);

        UpdateMouse();

        if (Input.MenuCancel.Pressed || MInput.Keyboard.Pressed(Keys.Escape)) {
            Back();
        } else if (Input.MenuConfirm.Pressed || MInput.Keyboard.Pressed(Keys.Enter)
            || MInput.Keyboard.Pressed(Keys.Space) || MInput.Keyboard.Pressed(Keys.L)) {
            ActivateFocused();
        }
    }

    private void UpdateMouse() {
        Vector2 mouse = MousePositionInUi();
        Point mousePoint = new((int) mouse.X, (int) mouse.Y);
        for (int index = 0; index < model.ProjectCount; index++) {
            if (!IsProjectVisible(index)) continue;
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

    private void ScrollColumns(int amount) {
        firstVisibleColumn = Math.Clamp(firstVisibleColumn + amount, 0, MaxFirstVisibleColumn);
    }

    private void EnsureFocusedVisible() {
        if (model.Focus != TrainingMenuFocus.Project) return;
        int column = model.ProjectColumn;
        if (column < firstVisibleColumn) firstVisibleColumn = column;
        else if (column >= firstVisibleColumn + VisibleColumns) firstVisibleColumn = column - VisibleColumns + 1;
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
        Draw.Rect(0f, 0f, UiWidth, UiHeight, Overlay);
        Draw.Rect(0f, 0f, UiWidth, 10f, Primary);
        ChineseText.Draw("CELESTE 训练场", new Vector2(120f, 86f), Vector2.Zero, 1.2f, Color.White, 4f);
        ChineseText.Draw("选择一个训练地图", new Vector2(122f, 174f), Vector2.Zero, 0.65f, new Color(176, 194, 222), 3f);

        for (int index = 0; index < model.ProjectCount; index++) {
            if (IsProjectVisible(index)) RenderProject(index);
        }
        RenderScrollAffordance();
        RenderButton(StartBounds, "开始训练", model.Focus == TrainingMenuFocus.Start, Primary);
        RenderButton(BackBounds, "返回", model.Focus == TrainingMenuFocus.Back, Card);
        ChineseText.Draw("WASD / 方向键选择    鼠标滚轮浏览    L / Enter 确认    Esc 返回", new Vector2(120f, 1016f), new Vector2(0f, 1f), 0.42f, new Color(173, 188, 213), 2f);
        RenderMouseCursor(MousePositionInUi());
    }

    private void RenderProject(int index) {
        Rectangle bounds = ProjectBounds(index);
        bool selected = model.ProjectIndex == index;
        bool focused = selected && model.Focus == TrainingMenuFocus.Project;
        Draw.Rect(bounds.X + 10, bounds.Y + 14, bounds.Width, bounds.Height, Color.Black * 0.55f);
        Draw.Rect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? new Color(30, 54, 72, 248) : Card * 0.96f);
        Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? Highlight : new Color(76, 91, 120));
        TrainingProjectOption project = model.ProjectAt(index);
        RenderPlaceholder(new Rectangle(bounds.X + 16, bounds.Y + 16, bounds.Width - 32, 148), index, focused);
        Draw.Rect(bounds.X + 20, bounds.Y + 178, 82, 30, focused ? Highlight : new Color(69, 91, 125));
        ChineseText.Draw(project.Difficulty, new Vector2(bounds.X + 61f, bounds.Y + 193f), new Vector2(0.5f, 0.5f), 0.38f, Color.White, 2f);
        ChineseText.Draw(project.Title, new Vector2(bounds.X + 20f, bounds.Y + 218f), Vector2.Zero, 0.58f, Color.White, 3f);
        ChineseText.Draw(project.Summary, new Vector2(bounds.X + 21f, bounds.Y + 258f), new Vector2(0f, 0.5f), 0.32f, new Color(190, 205, 228), 2f);
        if (focused) ChineseText.Draw("L / Enter 开始", new Vector2(bounds.Right - 20f, bounds.Bottom - 18f), Vector2.One, 0.34f, Highlight, 2f);
    }

    private void RenderScrollAffordance() {
        int columnCount = (model.ProjectCount + TrainingMenuModel.Rows - 1) / TrainingMenuModel.Rows;
        if (columnCount <= VisibleColumns) return;
        if (firstVisibleColumn > 0) {
            ChineseText.Draw("‹", new Vector2(82f, 531f), new Vector2(0.5f, 0.5f), 1.15f, Highlight, 3f);
        }
        if (firstVisibleColumn < MaxFirstVisibleColumn) {
            ChineseText.Draw("›", new Vector2(1838f, 531f), new Vector2(0.5f, 0.5f), 1.15f, Highlight, 3f);
        }
        ChineseText.Draw($"{firstVisibleColumn + 1}–{Math.Min(firstVisibleColumn + VisibleColumns, columnCount)} / {columnCount} 列",
            new Vector2(1798f, 1016f), new Vector2(1f, 1f), 0.34f, new Color(173, 188, 213), 2f);
    }

    private static void RenderPlaceholder(Rectangle bounds, int index, bool focused) {
        Color sky = focused ? new Color(46, 109, 133) : new Color(39, 70, 102);
        Draw.Rect(bounds, sky);
        Draw.Rect(bounds.X, bounds.Bottom - 58, bounds.Width, 58, new Color(19, 33, 53));
        Color mountain = index % 2 == 0 ? new Color(237, 100, 124) : new Color(126, 105, 216);
        Vector2 peak = new(bounds.Center.X, bounds.Y + 42);
        for (int row = 0; row < 8; row++) {
            int width = 28 + row * 34;
            Draw.Rect(peak.X - width / 2f, peak.Y + row * 17f, width, 19f, mountain * (1f - row * 0.035f));
        }
        Draw.Circle(new Vector2(bounds.Right - 58, bounds.Y + 52), 18f, new Color(255, 224, 166), 16);
        ChineseText.Draw("地图预览", new Vector2(bounds.X + 18f, bounds.Bottom - 19f), new Vector2(0f, 1f), 0.38f, Color.White, 2f);
    }

    private static void RenderButton(Rectangle bounds, string label, bool focused, Color color) {
        Draw.Rect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? color : color * 0.65f);
        Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? Color.White : new Color(82, 95, 119));
        ChineseText.Draw(label, bounds.Center.ToVector2(), new Vector2(0.5f, 0.5f), 0.55f, Color.White, 3f);
    }

    private static void RenderMouseCursor(Vector2 mouse) {
        Color shadow = Color.Black * 0.8f;
        Draw.Line(mouse + new Vector2(4f, 5f), mouse + new Vector2(4f, 39f), shadow, 9f);
        Draw.Line(mouse + new Vector2(4f, 5f), mouse + new Vector2(27f, 29f), shadow, 9f);
        Draw.Line(mouse + new Vector2(4f, 39f), mouse + new Vector2(13f, 30f), shadow, 9f);
        Draw.Line(mouse, mouse + new Vector2(0f, 34f), Color.White, 5f);
        Draw.Line(mouse, mouse + new Vector2(23f, 24f), Color.White, 5f);
        Draw.Line(mouse + new Vector2(0f, 34f), mouse + new Vector2(10f, 24f), Color.White, 5f);
    }

    private bool IsProjectVisible(int index) {
        int column = index / TrainingMenuModel.Rows;
        return column >= firstVisibleColumn && column < firstVisibleColumn + VisibleColumns;
    }

    private Rectangle ProjectBounds(int index) {
        int column = index / TrainingMenuModel.Rows - firstVisibleColumn;
        int row = index % TrainingMenuModel.Rows;
        return new Rectangle(
            CardX + column * (CardWidth + CardColumnGap),
            CardTop + row * (CardHeight + CardRowGap),
            CardWidth,
            CardHeight
        );
    }

    private int MaxFirstVisibleColumn {
        get {
            int columnCount = (model.ProjectCount + TrainingMenuModel.Rows - 1) / TrainingMenuModel.Rows;
            return Math.Max(0, columnCount - VisibleColumns);
        }
    }

    private static Rectangle StartBounds => new(120, 882, 250, 76);
    private static Rectangle BackBounds => new(394, 882, 150, 76);
}
