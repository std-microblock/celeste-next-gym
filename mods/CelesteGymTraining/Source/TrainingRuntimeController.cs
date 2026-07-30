using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Input;
using Monocle;
using System.Text.Json;
using Celeste.Mod.Entities;

namespace Celeste.Mod.CelesteGymTraining;

public enum TrainingLessonStage {
    None,
    Demo,
    Assisted,
    Free
}

internal enum TrainingPanel {
    None,
    DemoComplete,
    Failure,
    AssistedComplete,
    LessonComplete,
    LevelComplete,
    Error
}

internal enum TrainingOutcomeKind {
    None,
    Success,
    Failure
}

[Tracked]
public sealed class TrainingRuntimeController : Entity {
    private const float UiWidth = 1920f;
    private const float UiHeight = 1080f;
    private static readonly Color Ink = new(8, 13, 25, 235);
    private static readonly Color Card = new(28, 39, 61, 245);
    private static readonly Color Cyan = new(91, 211, 219);
    private static readonly Color Pink = new(242, 91, 119);
    private static readonly Color Green = new(104, 222, 157);
    private static readonly Color Yellow = new(255, 207, 99);

    private readonly Level level;
    private readonly TrainingRuntimeProject project;
    private readonly HashSet<string> completed = new(StringComparer.Ordinal);
    private readonly Dictionary<string, TrainingFuzzResult> fuzzByLesson = new(StringComparer.Ordinal);
    private readonly TimeRateModifier timeRateModifier = new(1f, true);
    private string? nearbyLessonId;
    private TrainingLessonDefinition? lesson;
    private TrainingFuzzResult? fuzz;
    private TrainingLessonStage stage;
    private TrainingPanel panel;
    private List<TrainingFuzzCandidate> candidates = [];
    private List<TrainingFuzzInput> verifiedSequence = [];
    private readonly List<(int Frame, IReadOnlyList<string> Keys)> actualInputs = [];
    private int nextVerified;
    private int attemptFrame;
    private int demoFrame;
    private int panelFocus;
    private string failureTitle = "";
    private string failureBody = "";
    private string toast = "";
    private float toastTimer;
    private FrameButtons previousButtons;
    private bool guidedMode;
    private bool passiveSettled;
    private bool passiveSucceeded;
    private float passiveFeedbackTimer;
    private int? failureFrame;
    private float hudX = 80f;
    private bool hudOnRight;
    private readonly Dictionary<VirtualButton, List<VirtualButton.Node>> originalButtonNodes = [];
    private FrameButtons demoButtons;
    private FrameButtons demoPreviousButtons;
    private bool demoPaused;
    private int demoStepIndex;
    private bool demoReplaying;
    private int demoReplayTargetStep;
    private int demoControlFocus = 1;
    private TrainingOutcomeKind outcomeKind;
    private TrainingPanel pendingOutcomePanel;
    private float outcomeTimer;
    private float outcomeDuration;
    private float stageTransitionTimer;
    private TrainingLessonStage pendingStage;
    private float successToastTimer;
    private string successToastTitle = "";
    private string successToastBody = "";
    private int preEntryFrames;
    private int resultReactionFrames;
    private int? resultActualFrame;
    private int? resultTargetFrame;
    private float resultAccuracy;
    private IReadOnlyList<double> resultObjectiveValues = [];
    private IReadOnlyList<double> resultBestObjectiveValues = [];

    public TrainingRuntimeController(Level level, TrainingRuntimeProject project) {
        this.level = level;
        this.project = project;
        Tag = Tags.HUD | Tags.PauseUpdate | Tags.FrozenUpdate | Tags.TransitionUpdate;
        Depth = int.MinValue + 10;
        Add(timeRateModifier);
        foreach (string lessonId in project.Lessons.Keys) {
            if (level.Session.GetFlag(CompletionFlag(lessonId))) completed.Add(lessonId);
        }
    }

    public override void Added(Scene scene) {
        base.Added(scene);
        ChineseText.Prepare();
        try {
            TrainingNative.CacheMap(project.Map);
        } catch (Exception error) {
            ShowError("原生训练引擎加载失败", error.Message);
        }
    }

    public override void Removed(Scene scene) {
        RestoreDemoInput();
        RestoreTimeRate();
        base.Removed(scene);
    }

    public void SetNearbyLesson(string lessonId, bool inside) {
        if (!project.Lessons.ContainsKey(lessonId)) return;
        if (inside) {
            nearbyLessonId = lessonId;
            if (!guidedMode && !string.Equals(lesson?.Id, lessonId, StringComparison.Ordinal)) {
                BeginPassiveLesson(lessonId);
            }
        } else if (string.Equals(nearbyLessonId, lessonId, StringComparison.Ordinal)) {
            nearbyLessonId = null;
        }
    }

    public void ReachLessonEnd(string lessonId) {
        if (lesson is null || !string.Equals(lesson.Id, lessonId, StringComparison.Ordinal)) return;
        if (guidedMode) return;
        // Match the Web runtime: the end trigger commits an already successful
        // attempt, but merely reaching it never manufactures a failure.
        ResetMonitoredLesson();
    }

    public void ReachFinish(string projectId, bool requireAllModules) {
        if (!string.Equals(project.Id, projectId, StringComparison.Ordinal) || panel == TrainingPanel.LevelComplete) return;
        int missing = project.Lessons.Count - completed.Count;
        if (requireAllModules && missing > 0) {
            toast = $"还有 {missing} 个练习点未完成";
            toastTimer = 3f;
            return;
        }
        lesson = null;
        RestoreDemoInput();
        stage = TrainingLessonStage.None;
        panel = TrainingPanel.LevelComplete;
        panelFocus = 0;
        timeRateModifier.Multiplier = 0f;
    }

    public override void Update() {
        base.Update();
        if (toastTimer > 0f) toastTimer -= Engine.RawDeltaTime;
        if (successToastTimer > 0f) successToastTimer -= Engine.RawDeltaTime;
        UpdateHudPlacement();

        if (stageTransitionTimer > 0f) {
            stageTransitionTimer -= Engine.RawDeltaTime;
            if (stageTransitionTimer <= 0f) BeginStage(pendingStage);
            return;
        }

        if (outcomeKind != TrainingOutcomeKind.None) {
            UpdateOutcomeAnimation();
            return;
        }

        if (panel != TrainingPanel.None) {
            UpdatePanel();
            return;
        }
        if (lesson is null) return;
        if (!guidedMode) {
            UpdatePassiveAttempt();
            return;
        }
        if (MInput.Keyboard.Pressed(Keys.Back)) {
            CloseLesson();
            return;
        }
        if (stage == TrainingLessonStage.Demo) UpdateDemo();
        else UpdateAttempt();
    }

    private void UpdateNearbyPrompt() {
        if (nearbyLessonId is null) return;
        Rectangle launch = LaunchBoundsAt((int) hudX);
        bool hovered = launch.Contains(MousePointInUi());
        if (MInput.Keyboard.Pressed(Keys.D4)
            || MInput.Keyboard.Pressed(Keys.NumPad4)
            || hovered && MInput.Mouse.PressedLeftButton) {
            BeginLesson(nearbyLessonId);
        }
    }

    private void BeginPassiveLesson(string lessonId) {
        if (!project.Lessons.TryGetValue(lessonId, out TrainingLessonDefinition? selected)) return;
        try {
            lesson = selected;
            fuzz = fuzzByLesson.GetValueOrDefault(lessonId);
            if (fuzz is null) {
                fuzz = TrainingFuzzResult.Parse(TrainingNative.FuzzSearch(selected.InitialSnapshot, selected.Fuzz));
                fuzzByLesson.Add(lessonId, fuzz);
            }
            guidedMode = false;
            stage = TrainingLessonStage.None;
            ResetPassiveAttempt();
        } catch (Exception error) {
            ShowError("Fuzz 计算失败", error.Message);
        }
    }

    private void UpdatePassiveAttempt() {
        if (lesson is null || fuzz is null) return;
        if (MInput.Keyboard.Pressed(Keys.D4) || MInput.Keyboard.Pressed(Keys.NumPad4)
            || LaunchBoundsAt((int) hudX).Contains(MousePointInUi()) && MInput.Mouse.PressedLeftButton) {
            BeginLesson(lesson.Id);
            return;
        }
        if (passiveSettled) {
            passiveFeedbackTimer -= Engine.RawDeltaTime;
            previousButtons = CaptureButtons();
            if (passiveFeedbackTimer <= 0f && !AnyActionHeld(previousButtons)) ResetPassiveAttempt();
            return;
        }
        UpdateAttempt();
    }

    private void ResetPassiveAttempt() {
        if (lesson is null || fuzz is null) return;
        candidates = [.. fuzz.Candidates];
        verifiedSequence = VerifiedSequence(lesson);
        nextVerified = 0;
        attemptFrame = 0;
        actualInputs.Clear();
        previousButtons = CaptureButtons();
        passiveSettled = false;
        passiveSucceeded = false;
        passiveFeedbackTimer = 0f;
        preEntryFrames = 0;
        failureFrame = null;
        failureTitle = "";
        failureBody = "";
        RestoreTimeRate();
    }

    private void BeginLesson(string lessonId) {
        if (!project.Lessons.TryGetValue(lessonId, out TrainingLessonDefinition? selected)) return;
        try {
            lesson = selected;
            fuzz = fuzzByLesson.GetValueOrDefault(lessonId);
            if (fuzz is null) {
                fuzz = TrainingFuzzResult.Parse(TrainingNative.FuzzSearch(selected.InitialSnapshot, selected.Fuzz));
                fuzzByLesson.Add(lessonId, fuzz);
            }
            guidedMode = true;
            BeginStage(TrainingLessonStage.Demo);
        } catch (Exception error) {
            ShowError("Fuzz 计算失败", error.Message);
        }
    }

    private void BeginStage(TrainingLessonStage next) {
        if (lesson is null || fuzz is null) return;
        stage = next;
        panel = TrainingPanel.None;
        panelFocus = 0;
        candidates = [.. fuzz.Candidates];
        verifiedSequence = VerifiedSequence(lesson);
        nextVerified = 0;
        attemptFrame = 0;
        demoFrame = 0;
        actualInputs.Clear();
        previousButtons = default;
        passiveSettled = false;
        failureFrame = null;
        preEntryFrames = 0;
        RestoreTimeRate();
        ApplyInitialSnapshot();
        if (next == TrainingLessonStage.Demo) {
            demoPaused = true;
            demoStepIndex = 0;
            demoReplaying = false;
            demoReplayTargetStep = 0;
            demoControlFocus = 1;
            demoButtons = default;
            demoPreviousButtons = default;
            InstallDemoInput();
            timeRateModifier.Multiplier = 0f;
        } else {
            RestoreDemoInput();
        }
    }

    private void UpdateDemo() {
        if (lesson is null || fuzz is null) return;
        if (!demoPaused) return;
        bool previous = Input.MenuLeft.Pressed || MInput.Keyboard.Pressed(Keys.A) || MInput.Keyboard.Pressed(Keys.Left);
        bool next = Input.MenuRight.Pressed || MInput.Keyboard.Pressed(Keys.D) || MInput.Keyboard.Pressed(Keys.Right);
        if (previous) demoControlFocus = 0;
        else if (next) demoControlFocus = 1;
        Point mouse = MousePointInUi();
        for (int index = 0; index < 2; index++) {
            if (!DemoButtonBounds(index).Contains(mouse)) continue;
            demoControlFocus = index;
            if (MInput.Mouse.PressedLeftButton) ActivateDemoControl(index);
            return;
        }
        if (Input.MenuConfirm.Pressed || MInput.Keyboard.Pressed(Keys.Enter) || MInput.Keyboard.Pressed(Keys.L)) {
            ActivateDemoControl(demoControlFocus);
        }
    }

    private void ActivateDemoControl(int index) {
        if (index == 0) {
            if (demoStepIndex <= 0) return;
            int targetStep = demoStepIndex - 1;
            ApplyInitialSnapshot();
            demoFrame = 0;
            demoStepIndex = 0;
            demoReplayTargetStep = targetStep;
            demoReplaying = targetStep > 0;
            demoPaused = !demoReplaying;
            demoButtons = default;
            demoPreviousButtons = default;
            timeRateModifier.Multiplier = demoPaused ? 0f : 0.65f;
            return;
        }
        demoReplaying = false;
        demoPaused = false;
        timeRateModifier.Multiplier = 0.65f;
    }

    private void UpdateAttempt() {
        if (lesson is null || fuzz is null || nextVerified >= verifiedSequence.Count) return;
        Player? player = level.Tracker.GetEntity<Player>();
        FrameButtons current = CaptureButtons();
        TrainingFuzzInput expected = verifiedSequence[nextVerified];
        bool expectedTriggered = ExpectedTriggered(current, previousButtons, expected.Keys);
        bool otherActionTriggered = AnyActionTriggered(current, previousButtons);
        if (expectedTriggered || otherActionTriggered) {
            IReadOnlyList<string> actual = VerificationKeys(current, previousButtons, expected.Keys);
            bool entryCheckPassed = nextVerified != 0
                || player is not null && TrainingNative.EvaluateEntryChecks(CaptureSnapshot(player), lesson.EntryChecks);
            VerifyInput(expected, actual, entryCheckPassed);
        } else if (nextVerified > 0) {
            int latest = CandidateFrames(expected.Index).DefaultIfEmpty(-1).Max();
            if (latest >= 0 && attemptFrame > latest) {
                FailWindow(expected);
            }
        }
        if (!passiveSettled && panel == TrainingPanel.None && stage == TrainingLessonStage.Assisted) ApplyAssistedTimeScale(expected);
        previousButtons = current;
        if (nextVerified > 0) attemptFrame++;
        else preEntryFrames++;
    }

    private void VerifyInput(TrainingFuzzInput expected, IReadOnlyList<string> actual, bool entryCheckPassed) {
        if (lesson is null) return;
        if (!SameKeys(actual, expected.Keys) || !entryCheckPassed) {
            if (nextVerified == 0) Fail(lesson.EntryFailureTitle, lesson.EntryFailureBody);
            else {
                TrainingTeachingStep step = CurrentStep;
                Fail(step.OrderErrorTitle, step.OrderErrorBody);
            }
            return;
        }
        int frame = nextVerified == 0 ? 0 : attemptFrame;
        List<TrainingFuzzCandidate> matching = candidates.Where(candidate =>
            candidate.VerifiedInputs.Any(input =>
                input.InputIndex == expected.Index
                && input.Frame == frame
                && SameKeys(input.Keys, actual)
            )
        ).ToList();
        if (matching.Count == 0) {
            FailWindow(expected);
            return;
        }
        candidates = matching;
        actualInputs.Add((frame, actual));
        nextVerified++;
        if (nextVerified == 1) attemptFrame = 0;
        if (nextVerified >= verifiedSequence.Count) CompleteAttempt();
    }

    private void CompleteAttempt() {
        RestoreTimeRate();
        if (!guidedMode) {
            CaptureResultStats(true);
            if (lesson is not null) {
                completed.Add(lesson.Id);
                level.Session.SetFlag(CompletionFlag(lesson.Id));
            }
            passiveSettled = true;
            passiveSucceeded = true;
            passiveFeedbackTimer = 2.4f;
            ShowSuccessToast("动作正确", "已自动记录；继续游戏即可。");
            return;
        }
        CaptureResultStats(true);
        if (stage == TrainingLessonStage.Assisted) {
            ShowSuccessToast("辅助实操完成", "停留一秒查看结果，随后进入自由练习。");
            pendingStage = TrainingLessonStage.Free;
            stageTransitionTimer = 1f;
            timeRateModifier.Multiplier = 0f;
        } else {
            if (lesson is not null) {
                completed.Add(lesson.Id);
                level.Session.SetFlag(CompletionFlag(lesson.Id));
            }
            ShowSuccessToast("MODULE CLEAR", "动作正确；已自动记录，继续游戏即可。");
            guidedMode = false;
            stage = TrainingLessonStage.None;
            passiveSettled = true;
            passiveSucceeded = true;
            passiveFeedbackTimer = 2.4f;
        }
    }

    private void FailWindow(TrainingFuzzInput expected) {
        TrainingTeachingStep step = CurrentStep;
        int[] frames = CandidateFrames(expected.Index).Distinct().Order().ToArray();
        string window = frames.Length == 0 ? "" : $" 可行帧：{CompressFrames(frames)}";
        Fail(step.WindowErrorTitle, step.WindowErrorBody + window);
    }

    private void Fail(string title, string body) {
        failureTitle = string.IsNullOrWhiteSpace(title) ? "本次动作未通过" : title;
        failureBody = string.IsNullOrWhiteSpace(body)
            ? "输入顺序或时机与当前 Fuzz 窗口不匹配，请查看 Timeline 后重试。"
            : body;
        failureFrame = attemptFrame;
        RestoreTimeRate();
        if (!guidedMode) {
            CaptureResultStats(false);
            passiveSettled = true;
            passiveSucceeded = false;
            passiveFeedbackTimer = float.PositiveInfinity;
            BeginOutcome(TrainingPanel.Failure, false);
            return;
        }
        BeginOutcome(TrainingPanel.Failure, false);
    }

    private void ApplyAssistedTimeScale(TrainingFuzzInput expected) {
        if (lesson is null || !lesson.SlowdownEnabled || nextVerified == 0) {
            timeRateModifier.Multiplier = 1f;
            return;
        }
        int target = CandidateFrames(expected.Index).Where(frame => frame >= attemptFrame).DefaultIfEmpty(-1).Min();
        if (target < 0 || lesson.SlowdownRadiusFrames <= 0) {
            timeRateModifier.Multiplier = 1f;
            return;
        }
        float distance = Math.Max(target - attemptFrame, 0);
        float progress = MathHelper.Clamp(1f - distance / lesson.SlowdownRadiusFrames, 0f, 1f);
        float multiplier = MathHelper.Lerp(1f, lesson.MinimumTimeMultiplier, progress);
        timeRateModifier.Multiplier = multiplier;
    }

    private void UpdatePanel() {
        string[] buttons = PanelButtons;
        if (buttons.Length == 0) return;
        bool previous = Input.MenuLeft.Pressed || Input.MenuUp.Pressed
            || MInput.Keyboard.Pressed(Keys.A) || MInput.Keyboard.Pressed(Keys.W)
            || MInput.Keyboard.Pressed(Keys.Left) || MInput.Keyboard.Pressed(Keys.Up);
        bool next = Input.MenuRight.Pressed || Input.MenuDown.Pressed
            || MInput.Keyboard.Pressed(Keys.D) || MInput.Keyboard.Pressed(Keys.S)
            || MInput.Keyboard.Pressed(Keys.Right) || MInput.Keyboard.Pressed(Keys.Down);
        if (previous) panelFocus = (panelFocus - 1 + buttons.Length) % buttons.Length;
        else if (next) panelFocus = (panelFocus + 1) % buttons.Length;

        Point mouse = MousePointInUi();
        for (int index = 0; index < buttons.Length; index++) {
            if (!PanelButtonBounds(index, buttons.Length).Contains(mouse)) continue;
            panelFocus = index;
            if (MInput.Mouse.PressedLeftButton) ActivatePanelButton(index);
            return;
        }
        if (Input.MenuConfirm.Pressed || MInput.Keyboard.Pressed(Keys.Enter) || MInput.Keyboard.Pressed(Keys.L)) {
            ActivatePanelButton(panelFocus);
        }
    }

    private void ActivatePanelButton(int index) {
        switch (panel) {
            case TrainingPanel.DemoComplete:
                if (index == 0) BeginStage(TrainingLessonStage.Assisted); else CloseLesson();
                break;
            case TrainingPanel.Failure:
                if (index == 0) {
                    if (guidedMode) BeginStage(stage);
                    else {
                        panel = TrainingPanel.None;
                        RestoreTimeRate();
                        ApplyInitialSnapshot();
                        ResetPassiveAttempt();
                    }
                } else CloseLesson();
                break;
            case TrainingPanel.AssistedComplete:
                if (index == 0) BeginStage(TrainingLessonStage.Free);
                else if (index == 1) BeginStage(TrainingLessonStage.Assisted);
                else CloseLesson();
                break;
            case TrainingPanel.LessonComplete:
                if (index == 0) CloseLesson();
                else if (index == 1) BeginStage(TrainingLessonStage.Free);
                else CloseLesson();
                break;
            case TrainingPanel.LevelComplete:
                if (index == 0) ReturnToProjectMenu();
                else DismissPanel();
                break;
            case TrainingPanel.Error:
                DismissPanel();
                break;
        }
    }

    internal bool TryConsumeQuickRestart() {
        if (lesson is null || !Input.QuickRestart.Pressed) return false;
        Input.QuickRestart.ConsumeBuffer();
        return true;
    }

    internal void RetryCurrentStageFromBinding() => RetryCurrentStage();

    private void RetryCurrentStage() {
        if (lesson is null || fuzz is null) return;
        outcomeKind = TrainingOutcomeKind.None;
        pendingOutcomePanel = TrainingPanel.None;
        outcomeTimer = 0f;
        stageTransitionTimer = 0f;
        panel = TrainingPanel.None;
        panelFocus = 0;
        successToastTimer = 0f;
        toastTimer = 0f;
        failureTitle = "";
        failureBody = "";
        failureFrame = null;
        RestoreDemoInput();
        RestoreTimeRate();
        if (guidedMode) {
            BeginStage(stage == TrainingLessonStage.None ? TrainingLessonStage.Demo : stage);
        } else {
            ApplyInitialSnapshot();
            ResetPassiveAttempt();
        }
        toast = "已恢复到本段起点";
        toastTimer = 1.8f;
    }

    private void CloseLesson() {
        RestoreDemoInput();
        RestoreTimeRate();
        guidedMode = false;
        stage = TrainingLessonStage.None;
        panel = TrainingPanel.None;
        if (nearbyLessonId is not null) BeginPassiveLesson(nearbyLessonId);
        else ResetMonitoredLesson();
    }

    private void ResetMonitoredLesson() {
        lesson = null;
        fuzz = null;
        candidates.Clear();
        verifiedSequence.Clear();
        actualInputs.Clear();
        nextVerified = 0;
        attemptFrame = 0;
        passiveSettled = false;
        failureFrame = null;
    }

    private void DismissPanel() {
        RestoreTimeRate();
        panel = TrainingPanel.None;
    }

    private void ReturnToProjectMenu() {
        RestoreDemoInput();
        RestoreTimeRate();
        level.Session.SetFlag(CelesteGymTrainingModule.TrainingActiveFlag, false);
        level.Session.RespawnPoint = null;
        Engine.Scene = new LevelLoader(level.Session) { PlayerIntroTypeOverride = Player.IntroTypes.None };
    }

    private void ShowError(string title, string body) {
        RestoreDemoInput();
        failureTitle = title;
        failureBody = body;
        panel = TrainingPanel.Error;
        panelFocus = 0;
        timeRateModifier.Multiplier = 0f;
    }

    private void RestoreTimeRate() {
        timeRateModifier.Multiplier = 1f;
    }

    private void BeginOutcome(TrainingPanel targetPanel, bool success) {
        CaptureResultStats(success);
        pendingOutcomePanel = targetPanel;
        outcomeKind = success ? TrainingOutcomeKind.Success : TrainingOutcomeKind.Failure;
        outcomeTimer = 0f;
        outcomeDuration = success ? 0.8f : 1f;
        panel = TrainingPanel.None;
        timeRateModifier.Multiplier = 1f;
    }

    private void ShowSuccessToast(string title, string body) {
        successToastTitle = title;
        successToastBody = body;
        successToastTimer = 3.2f;
    }

    private void UpdateOutcomeAnimation() {
        outcomeTimer += Engine.RawDeltaTime;
        float progress = MathHelper.Clamp(outcomeTimer / Math.Max(0.01f, outcomeDuration), 0f, 1f);
        float smooth = progress * progress * (3f - 2f * progress);
        timeRateModifier.Multiplier = 1f - smooth;
        if (progress < 1f) return;
        outcomeKind = TrainingOutcomeKind.None;
        panel = pendingOutcomePanel;
        panelFocus = 0;
        timeRateModifier.Multiplier = 0f;
    }

    private void CaptureResultStats(bool success) {
        if (fuzz is null) return;
        TrainingFuzzCandidate? actual = success
            ? candidates.FirstOrDefault()
            : fuzz.Evaluations.FirstOrDefault(MatchesActualPrefix);
        resultObjectiveValues = actual?.ObjectiveValues ?? [];
        resultBestObjectiveValues = fuzz.Best.ObjectiveValues;
        resultAccuracy = resultObjectiveValues.Count == 0 || resultBestObjectiveValues.Count == 0
            ? success ? 100f : 0f
            : OutputAccuracy(resultObjectiveValues[0], resultBestObjectiveValues[0]);
        resultReactionFrames = preEntryFrames;
        resultActualFrame = actualInputs.Count > 0 ? actualInputs[^1].Frame : failureFrame;
        int lastInput = verifiedSequence.Count == 0 ? -1 : verifiedSequence[^1].Index;
        resultTargetFrame = fuzz.Best.VerifiedInputs.FirstOrDefault(input => input.InputIndex == lastInput)?.Frame;
    }

    private static float OutputAccuracy(double actual, double best) {
        if (!double.IsFinite(actual) || !double.IsFinite(best)) return 0f;
        if (Math.Abs(best) < 0.000000001) return Math.Abs(actual - best) < 0.000000001 ? 100f : 0f;
        return MathHelper.Clamp((float) (1d - Math.Abs(actual - best) / Math.Abs(best)) * 100f, 0f, 100f);
    }

    public override void Render() {
        if (lesson is not null) {
            if (guidedMode) RenderLessonHud();
            else RenderPassiveHud();
            if (fuzz is not null) RenderBottomTimeline();
        }
        if (toastTimer > 0f) RenderToast();
        if (successToastTimer > 0f) RenderSuccessToast();
        if (outcomeKind != TrainingOutcomeKind.None) RenderOutcomeAnimation();
        if (panel != TrainingPanel.None) RenderPanel();
        if (lesson is not null || panel != TrainingPanel.None || nearbyLessonId is not null) {
            RenderMouseCursor(MousePositionInUi());
        }
    }

    private void RenderOutcomeAnimation() {
        float progress = MathHelper.Clamp(outcomeTimer / Math.Max(0.01f, outcomeDuration), 0f, 1f);
        RenderFailurePopup(progress);
    }

    private void RenderPassiveHud() {
        if (lesson is null) return;
        Rectangle box = new((int) hudX, 54, 620, 176);
        Color accent = Cyan;
        Draw.Rect(box, Ink);
        Draw.HollowRect(box.X, box.Y, box.Width, box.Height, accent);
        ChineseText.Draw(lesson.Title, new Vector2(box.X + 22, box.Y + 18), Vector2.Zero, 0.48f, Color.White, 3f);
        ChineseText.Draw("自动判定", new Vector2(box.Right - 22, box.Y + 20), new Vector2(1f, 0f), 0.30f, accent, 2f);
        ChineseText.Draw(AttemptPrompt, new Vector2(box.X + 22, box.Y + 67), Vector2.Zero, 0.34f, Color.White, 2f);
        ChineseText.Draw(nextVerified == 0 ? "到达练习点后已开始实时判定" : $"当前 F{attemptFrame} · 结果见下方 TIMELINE",
            new Vector2(box.X + 23, box.Y + 111), Vector2.Zero, 0.25f, new Color(168, 187, 215), 2f);

        Rectangle launch = LaunchBoundsAt(box.X);
        bool hovered = launch.Contains(MousePointInUi());
        Draw.Rect(launch, hovered ? new Color(38, 72, 86, 245) : new Color(27, 42, 65, 245));
        Draw.HollowRect(launch.X, launch.Y, launch.Width, launch.Height, hovered ? Cyan : new Color(93, 112, 143));
        ChineseText.Draw("按 4 进入三阶段主动训练", launch.Center.ToVector2(), new Vector2(0.5f), 0.31f, Cyan, 2f);
    }

    private void RenderLessonHud() {
        if (lesson is null) return;
        Rectangle box = new((int) hudX, 54, 620, stage == TrainingLessonStage.Demo ? 250 : 204);
        Draw.Rect(box, Ink);
        Draw.HollowRect(box.X, box.Y, box.Width, box.Height, StageColor);
        ChineseText.Draw(lesson.Title, new Vector2(box.X + 22, box.Y + 16), Vector2.Zero, 0.48f, Color.White, 3f);
        ChineseText.Draw(StageLabel, new Vector2(box.Right - 22, box.Y + 19), new Vector2(1f, 0f), 0.30f, StageColor, 2f);
        RenderStageChips(box.X + 20, box.Y + 56, box.Width - 40);
        string prompt = stage == TrainingLessonStage.Demo ? DemoPrompt : AttemptPrompt;
        Draw.Rect(box.X + 20, box.Y + 106, box.Width - 40, 52, new Color(20, 31, 51, 245));
        Draw.HollowRect(box.X + 20, box.Y + 106, box.Width - 40, 52, StageColor);
        ChineseText.Draw(prompt, new Vector2(box.Center.X, box.Y + 132), new Vector2(0.5f), 0.30f, Color.White, 2f);
        if (stage == TrainingLessonStage.Demo) RenderDemoControls();
        ChineseText.Draw("Backspace 退出练习", new Vector2(box.Right - 20, box.Bottom - 12), new Vector2(1f, 1f), 0.23f, new Color(169, 187, 215), 2f);
    }

    private void RenderDemoControls() {
        IReadOnlyList<TrainingVerifiedInput> steps = DemoSteps;
        int count = Math.Max(1, steps.Count);
        string status = demoPaused
            ? $"演示步骤 {Math.Min(demoStepIndex + 1, count)}/{count}"
            : $"正在播放 · F{demoFrame}";
        ChineseText.Draw(status, new Vector2(hudX + 310, 176), new Vector2(0.5f, 0.5f), 0.23f, Yellow, 2f);
        if (!demoPaused) return;
        for (int index = 0; index < 2; index++) {
            Rectangle bounds = DemoButtonBounds(index);
            bool disabled = index == 0 && demoStepIndex <= 0;
            bool focused = demoControlFocus == index && !disabled;
            Color fill = disabled ? new Color(35, 42, 57) : focused ? Yellow : new Color(48, 57, 78);
            Draw.Rect(bounds, fill);
            Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? Color.White : new Color(103, 117, 143));
            ChineseText.Draw(index == 0 ? "上一步" : "执行本步", bounds.Center.ToVector2(), new Vector2(0.5f), 0.27f,
                disabled ? new Color(112, 123, 143) : focused ? new Color(28, 31, 39) : Color.White, 2f);
        }
    }

    private void RenderBottomTimeline() {
        RenderTimeline(new Rectangle(24, 808, 1872, 248), guidedMode);
        ChineseText.Draw("R  恢复本段起点", new Vector2(1874, 1034), new Vector2(1f, 1f), 0.22f, Yellow, 0f);
    }

    private void RenderSuccessToast() {
        const int width = 570;
        Rectangle box = new((int) hudX, 54, width, 262);
        float elapsed = 3.2f - successToastTimer;
        float enter = 1f - MathF.Pow(1f - MathHelper.Clamp(elapsed / 0.52f, 0f, 1f), 3f);
        int slide = (int) ((1f - enter) * (hudOnRight ? 90f : -90f));
        box.X += slide;
        Draw.Rect(box, new Color(9, 13, 24, 248));
        Draw.Rect(box.X, box.Y, 6f, box.Height, Green);
        Draw.HollowRect(box.X, box.Y, box.Width, box.Height, Green * 0.72f);
        Draw.Circle(new Vector2(box.X + 42, box.Y + 42), 19f, Green, 20);
        ChineseText.Draw("✓", new Vector2(box.X + 42, box.Y + 42), new Vector2(0.5f), 0.42f, new Color(7, 23, 15), 2f);
        ChineseText.Draw("MODULE CLEAR", new Vector2(box.X + 76, box.Y + 20), Vector2.Zero, 0.22f, Green, 2f);
        ChineseText.Draw(successToastTitle, new Vector2(box.X + 76, box.Y + 46), Vector2.Zero, 0.40f, Color.White, 3f);
        ChineseText.Draw($"{resultAccuracy:0}%", new Vector2(box.Right - 22, box.Y + 24), new Vector2(1f, 0f), 0.56f, Yellow, 3f);
        RenderTimeline(new Rectangle(box.X + 16, box.Y + 86, box.Width - 32, 126), guidedMode);
        ChineseText.Draw(successToastBody, new Vector2(box.X + 18, box.Bottom - 33), Vector2.Zero, 0.24f, new Color(205, 216, 229), 2f);
    }

    private void RenderStageChips(int x, int y, int width) {
        (TrainingLessonStage Stage, string Label)[] chips = [
            (TrainingLessonStage.Demo, "1 演示"),
            (TrainingLessonStage.Assisted, "2 辅助实操"),
            (TrainingLessonStage.Free, "3 自由练习")
        ];
        int chipWidth = (width - 20) / 3;
        for (int index = 0; index < chips.Length; index++) {
            Rectangle bounds = new(x + index * (chipWidth + 10), y, chipWidth, 38);
            bool active = stage == chips[index].Stage;
            Draw.Rect(bounds, active ? StageColor : new Color(38, 50, 75));
            ChineseText.Draw(chips[index].Label, bounds.Center.ToVector2(), new Vector2(0.5f), 0.35f, active ? Color.White : new Color(161, 177, 205), 2f);
        }
    }

    private void RenderTimeline(Rectangle bounds, bool guided) {
        if (lesson is null || fuzz is null) return;
        Color accent = guided ? StageColor : passiveSettled ? passiveSucceeded ? Green : Pink : Cyan;
        int currentFrame = guided && stage == TrainingLessonStage.Demo ? demoFrame : attemptFrame;
        int inputIndex = TimelineInputIndex(currentFrame);
        int targetFrame = TimelineTargetFrame(inputIndex) ?? currentFrame;
        int maximum = Math.Max(48, fuzz.Best.VerifiedInputs.Select(input => input.Frame).DefaultIfEmpty(0).Max() + 24);
        int viewportFrames = Math.Min(48, maximum);
        int focus = guided && stage == TrainingLessonStage.Demo ? targetFrame : currentFrame;
        int viewportStart = Math.Clamp(focus - viewportFrames / 2, 0, Math.Max(0, maximum - viewportFrames));
        int viewportEnd = viewportStart + viewportFrames;
        float left = bounds.X + 18;
        float right = bounds.Right - 18;
        float top = bounds.Y + 28;
        float bottom = bounds.Bottom - 38;
        float axisY = bottom - 12;
        float X(int frame) => MathHelper.Lerp(left, right, MathHelper.Clamp((frame - viewportStart) / (float) viewportFrames, 0f, 1f));

        Draw.Rect(bounds, new Color(10, 17, 31, 248));
        Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, new Color(68, 84, 112));
        ChineseText.Draw($"TIMELINE · F{viewportStart}–F{viewportEnd}", new Vector2(bounds.X + 15, bounds.Y + 8), Vector2.Zero, 0.23f, new Color(156, 177, 208), 1f);
        ChineseText.Draw($"F{currentFrame}", new Vector2(bounds.Right - 15, bounds.Y + 7), new Vector2(1f, 0f), 0.28f, Color.White, 2f);

        for (int frame = viewportStart; frame <= viewportEnd; frame += 4) {
            float x = X(frame);
            Draw.Line(new Vector2(x, top), new Vector2(x, axisY + 7), new Color(45, 57, 78), 1f);
            ChineseText.Draw(frame.ToString(), new Vector2(x, axisY + 10), new Vector2(0.5f, 0f), 0.18f, new Color(121, 143, 177), 1f);
        }

        IEnumerable<int> frames = CandidateFrames(inputIndex);
        foreach ((int from, int to) in Windows(frames)) {
            if (to < viewportStart || from > viewportEnd) continue;
            float x1 = X(Math.Max(from, viewportStart));
            float x2 = X(Math.Min(to + 1, viewportEnd));
            Draw.Rect(x1, top, Math.Max(4f, x2 - x1), axisY - top, accent * 0.18f);
            Draw.Rect(x1, axisY - 6, Math.Max(4f, x2 - x1), 8f, accent * 0.9f);
        }

        RenderObjectiveCurves(inputIndex, viewportStart, viewportEnd, left, right, top + 4, axisY - 12);
        Draw.Line(new Vector2(left, axisY), new Vector2(right, axisY), new Color(91, 108, 139), 2f);

        if (viewportStart == 0) {
            float x = X(0);
            Draw.Rect(x - 1, top, 2, axisY - top, Pink);
            ChineseText.Draw("F0", new Vector2(x + 5, top), Vector2.Zero, 0.19f, Pink, 1f);
        }
        foreach (TrainingVerifiedInput reference in fuzz.Best.VerifiedInputs.Where(input => input.Frame >= viewportStart && input.Frame <= viewportEnd)) {
            float x = X(reference.Frame);
            Draw.Circle(new Vector2(x, top + 9), 4f, new Color(255, 211, 122), 10);
        }
        if (targetFrame >= viewportStart && targetFrame <= viewportEnd) {
            float x = X(targetFrame);
            Draw.Circle(new Vector2(x, axisY - 20), 7f, Yellow, 4);
            ChineseText.Draw("最佳", new Vector2(x, axisY - 38), new Vector2(0.5f, 1f), 0.18f, Yellow, 1f);
        }
        foreach ((int frame, IReadOnlyList<string> keys) in actualInputs.Where(input => input.Frame >= viewportStart && input.Frame <= viewportEnd)) {
            float x = X(frame);
            Draw.Circle(new Vector2(x, axisY - 8), 6f, Green, 12);
            ChineseText.Draw(HumanKeys(keys), new Vector2(x, axisY - 19), new Vector2(0.5f, 1f), 0.17f, Green, 1f);
        }
        if (failureFrame is int failed && failed >= viewportStart && failed <= viewportEnd) {
            float x = X(failed);
            Draw.Line(new Vector2(x - 6, axisY - 15), new Vector2(x + 6, axisY - 3), Pink, 3f);
            Draw.Line(new Vector2(x + 6, axisY - 15), new Vector2(x - 6, axisY - 3), Pink, 3f);
        }
        float cursor = X(currentFrame);
        Draw.Rect(cursor - 1.5f, top, 3f, axisY - top + 5, Color.White);

        string objective = ObjectiveSummary(inputIndex);
        ChineseText.Draw("窗口", new Vector2(bounds.X + 16, bounds.Bottom - 25), Vector2.Zero, 0.18f, accent, 1f);
        ChineseText.Draw("● 你的输入", new Vector2(bounds.X + 92, bounds.Bottom - 25), Vector2.Zero, 0.18f, Green, 1f);
        ChineseText.Draw("◆ 最佳点", new Vector2(bounds.X + 210, bounds.Bottom - 25), Vector2.Zero, 0.18f, Yellow, 1f);
        if (objective.Length > 0) ChineseText.Draw(objective, new Vector2(bounds.Right - 14, bounds.Bottom - 25), new Vector2(1f, 0f), 0.18f, new Color(159, 180, 214), 1f);
    }

    private void RenderPanel() {
        if (panel == TrainingPanel.Failure) {
            RenderFailurePopup(1f);
            return;
        }
        Draw.Rect(0, 0, UiWidth, UiHeight, Color.Black * 0.52f);
        bool hasAttemptResult = panel is TrainingPanel.AssistedComplete or TrainingPanel.LessonComplete;
        Rectangle box = hasAttemptResult
            ? new Rectangle(280, 128, 1360, 824)
            : new Rectangle(360, 286, 1200, 480);
        Draw.Rect(box, Card);
        Draw.HollowRect(box.X, box.Y, box.Width, box.Height, PanelColor);
        if (hasAttemptResult) RenderAttemptResultPanel(box);
        else {
            ChineseText.Draw(PanelTitle, new Vector2(960, 350), new Vector2(0.5f, 0f), 0.8f, PanelColor, 4f);
            ChineseText.Draw(PanelBody, new Vector2(960, 448), new Vector2(0.5f, 0f), 0.46f, Color.White, 3f);
        }
        string[] buttons = PanelButtons;
        for (int index = 0; index < buttons.Length; index++) {
            Rectangle bounds = PanelButtonBounds(index, buttons.Length);
            bool focused = panelFocus == index;
            Draw.Rect(bounds, focused ? PanelColor : new Color(46, 58, 83));
            Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? Color.White : new Color(100, 116, 145));
            ChineseText.Draw(buttons[index], bounds.Center.ToVector2(), new Vector2(0.5f), 0.44f, Color.White, 3f);
        }
        ChineseText.Draw("WASD / 方向键选择    L / Enter 确认    鼠标点击", new Vector2(960, hasAttemptResult ? 925 : 730), new Vector2(0.5f, 1f), 0.32f, new Color(173, 190, 216), 2f);
    }

    private void RenderFailurePopup(float progress) {
        float enter = 1f - MathF.Pow(1f - MathHelper.Clamp(progress, 0f, 1f), 3f);
        float shake = progress < 0.42f ? MathF.Sin(progress * 92f) * (1f - progress / 0.42f) * 10f : 0f;
        Draw.Rect(0f, 0f, UiWidth, 808f, new Color(57, 10, 27) * (0.38f + 0.46f * enter));
        int width = 1120;
        int height = 626;
        int x = (int) ((UiWidth - width) * 0.5f + shake);
        int y = (int) MathHelper.Lerp(188f, 112f, enter);
        Rectangle card = new(x, y, width, height);
        Draw.Rect(card, new Color(11, 7, 25, 248));
        Draw.HollowRect(card.X, card.Y, card.Width, card.Height, Pink * (0.45f + 0.55f * enter));
        float sweepX = MathHelper.Lerp(card.X - 240, card.Right + 80, enter);
        Draw.Rect(sweepX, card.Y, 54f, card.Height, Pink * (0.08f * (1f - enter * 0.7f)));

        ChineseText.Draw("ATTEMPT RESULT", new Vector2(card.X + 30, card.Y + 24), Vector2.Zero, 0.24f, new Color(170, 135, 187), 2f);
        ChineseText.Draw(failureTitle, new Vector2(card.X + 30, card.Y + 56), Vector2.Zero, 0.58f, Color.White, 4f);
        ChineseText.Draw(ResultTimingLabel, new Vector2(card.Right - 30, card.Y + 68), new Vector2(1f, 0f), 0.34f, Yellow, 2f);
        RenderTimeline(new Rectangle(card.X + 28, card.Y + 124, card.Width - 56, 220), true);

        int statsY = card.Y + 366;
        int statWidth = 250;
        RenderResultStat(new Rectangle(card.X + 28, statsY, statWidth, 82), "输入时机", ResultTimingLabel, 0);
        RenderResultStat(new Rectangle(card.X + 292, statsY, statWidth, 82), "反应帧", $"{resultReactionFrames}F", 1);
        RenderResultStat(new Rectangle(card.X + 556, statsY, statWidth, 82), "实际 / 最佳", ResultFramePair, 2);
        RenderResultStat(new Rectangle(card.X + 820, statsY, 272, 82), "OBJECTIVE", ResultValue(resultObjectiveValues, 0), 3);
        ChineseText.Draw(failureBody, new Vector2(card.X + 30, card.Y + 475), Vector2.Zero, 0.30f, new Color(216, 203, 220), 2f);

        if (outcomeKind != TrainingOutcomeKind.None) {
            ChineseText.Draw("正在慢放失败帧…", new Vector2(card.Right - 30, card.Bottom - 30), new Vector2(1f, 1f), 0.24f, Pink, 2f);
            return;
        }
        string[] buttons = PanelButtons;
        for (int index = 0; index < buttons.Length; index++) {
            Rectangle bounds = PanelButtonBounds(index, buttons.Length);
            bool focused = panelFocus == index;
            Draw.Rect(bounds, focused ? Pink : new Color(46, 36, 59));
            Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, focused ? Color.White : new Color(128, 91, 112));
            ChineseText.Draw(buttons[index], bounds.Center.ToVector2(), new Vector2(0.5f), 0.38f, Color.White, 3f);
        }
        ChineseText.Draw("WASD / 方向键选择    L / Enter 确认    鼠标点击", new Vector2(card.Center.X, card.Bottom - 18),
            new Vector2(0.5f, 1f), 0.25f, new Color(173, 190, 216), 2f);
    }

    private void RenderAttemptResultPanel(Rectangle box) {
        bool success = panel is TrainingPanel.AssistedComplete or TrainingPanel.LessonComplete;
        ChineseText.Draw(success ? "MODULE CLEAR" : "ATTEMPT RESULT", new Vector2(box.X + 42, box.Y + 28),
            Vector2.Zero, 0.27f, PanelColor, 2f);
        ChineseText.Draw(PanelTitle, new Vector2(box.X + 42, box.Y + 64), Vector2.Zero, 0.66f, Color.White, 4f);
        ChineseText.Draw($"{resultAccuracy:0}%", new Vector2(box.Right - 42, box.Y + 45), new Vector2(1f, 0f), 0.86f, PanelColor, 4f);
        ChineseText.Draw("精准度", new Vector2(box.Right - 42, box.Y + 112), new Vector2(1f, 0f), 0.26f, new Color(166, 184, 213), 2f);

        RenderTimeline(new Rectangle(box.X + 40, box.Y + 150, box.Width - 80, 224), true);

        int statsTop = box.Y + 398;
        RenderResultStat(new Rectangle(box.X + 40, statsTop, 286, 92), "输入时机", ResultTimingLabel, 0);
        RenderResultStat(new Rectangle(box.X + 342, statsTop, 286, 92), "反应帧", $"{resultReactionFrames}F", 1);
        RenderResultStat(new Rectangle(box.X + 644, statsTop, 286, 92), "实际 / 最佳", ResultFramePair, 2);
        RenderResultStat(new Rectangle(box.X + 946, statsTop, 374, 92), "OBJECTIVE 准确度", $"{resultAccuracy:0.0}%", 3);

        int objectiveTop = box.Y + 512;
        int objectiveCount = Math.Min(3, Math.Max(resultObjectiveValues.Count, resultBestObjectiveValues.Count));
        if (objectiveCount == 0) {
            ChineseText.Draw(PanelBody, new Vector2(box.X + 42, objectiveTop + 24), Vector2.Zero, 0.34f, Color.White, 2f);
            return;
        }
        ChineseText.Draw("OBJECTIVE 对照", new Vector2(box.X + 42, objectiveTop), Vector2.Zero, 0.25f, new Color(156, 178, 211), 2f);
        int rowWidth = (box.Width - 104 - (objectiveCount - 1) * 18) / objectiveCount;
        for (int index = 0; index < objectiveCount; index++) {
            int x = box.X + 40 + index * (rowWidth + 18);
            Rectangle row = new(x, objectiveTop + 32, rowWidth, 78);
            Draw.Rect(row, new Color(19, 29, 48, 245));
            Draw.HollowRect(row.X, row.Y, row.Width, row.Height, new Color(65, 82, 111));
            string expression = lesson is not null && index < lesson.ObjectiveExpressions.Count
                ? lesson.ObjectiveExpressions[index]
                : $"OBJECTIVE {index + 1}";
            string actual = ResultValue(resultObjectiveValues, index);
            string best = ResultValue(resultBestObjectiveValues, index);
            ChineseText.Draw(expression, new Vector2(row.X + 16, row.Y + 12), Vector2.Zero, 0.22f, new Color(158, 179, 211), 1f);
            ChineseText.Draw($"本次 {actual}   /   最佳 {best}", new Vector2(row.X + 16, row.Y + 43), Vector2.Zero, 0.29f, Color.White, 2f);
        }
        ChineseText.Draw(PanelBody, new Vector2(box.X + 42, box.Bottom - 154), Vector2.Zero, 0.30f, Color.White, 2f);
    }

    private void RenderResultStat(Rectangle bounds, string label, string value, int index) {
        Color[] accents = [Cyan, Yellow, Green, Pink];
        Draw.Rect(bounds, new Color(19, 29, 48, 245));
        Draw.Rect(bounds.X, bounds.Y, 5f, bounds.Height, accents[Math.Clamp(index, 0, accents.Length - 1)]);
        ChineseText.Draw(label, new Vector2(bounds.X + 18, bounds.Y + 15), Vector2.Zero, 0.23f, new Color(155, 177, 210), 1f);
        ChineseText.Draw(value, new Vector2(bounds.X + 18, bounds.Y + 48), Vector2.Zero, 0.35f, Color.White, 2f);
    }

    private string ResultTimingLabel {
        get {
            if (resultActualFrame is null || resultTargetFrame is null) return "未命中";
            int difference = resultActualFrame.Value - resultTargetFrame.Value;
            if (difference == 0) return "正中最佳帧";
            return difference < 0 ? $"早 {-difference}F" : $"晚 {difference}F";
        }
    }

    private string ResultFramePair => $"{(resultActualFrame is int actual ? $"F{actual}" : "—")} / {(resultTargetFrame is int target ? $"F{target}" : "—")}";

    private static string ResultValue(IReadOnlyList<double> values, int index) =>
        index < values.Count && double.IsFinite(values[index]) ? values[index].ToString("0.##") : "—";

    private void RenderToast() {
        Rectangle bounds = new((int) hudX + 30, 514, 700, 66);
        Draw.Rect(bounds, Ink);
        Draw.HollowRect(bounds.X, bounds.Y, bounds.Width, bounds.Height, Yellow);
        ChineseText.Draw(toast, bounds.Center.ToVector2(), new Vector2(0.5f), 0.42f, Yellow, 3f);
    }

    private string DemoPrompt {
        get {
            if (lesson is null || fuzz is null) return "正在准备演示…";
            TrainingVerifiedInput? next = fuzz.Best.VerifiedInputs
                .Where(input => input.Frame >= demoFrame && verifiedSequence.Any(expected => expected.Index == input.InputIndex))
                .OrderBy(input => input.Frame)
                .FirstOrDefault();
            if (next is null) return "观察动作结果";
            TrainingFuzzInput? input = lesson.Inputs.FirstOrDefault(candidate => candidate.Index == next.InputIndex);
            return input is null ? "观察演示" : $"F{next.Frame}：{HumanKeys(input.Keys)}";
        }
    }

    private string AttemptPrompt => lesson is null || nextVerified >= verifiedSequence.Count
        ? "完成"
        : nextVerified == 0
            ? lesson.EntryHint
            : CurrentStep.Prompt;

    private TrainingTeachingStep CurrentStep => lesson?.Steps.Count > 0
        ? lesson.Steps[Math.Clamp(nextVerified, 0, lesson.Steps.Count - 1)]
        : new TrainingTeachingStep("按提示操作。", "动作顺序不正确", "请按提示操作。", "错过输入窗口", "请调整输入时机。");

    private string StageLabel => stage switch {
        TrainingLessonStage.Demo => "演示",
        TrainingLessonStage.Assisted => $"辅助实操 · {timeRateModifier.Multiplier:0.00}×",
        TrainingLessonStage.Free => "自由练习",
        _ => "训练"
    };

    private Color StageColor => stage switch {
        TrainingLessonStage.Demo => Cyan,
        TrainingLessonStage.Assisted => Yellow,
        TrainingLessonStage.Free => Green,
        _ => Cyan
    };

    private string PanelTitle => panel switch {
        TrainingPanel.DemoComplete => "演示完成",
        TrainingPanel.Failure => failureTitle,
        TrainingPanel.AssistedComplete => "辅助实操完成",
        TrainingPanel.LessonComplete => "本段训练完成",
        TrainingPanel.LevelComplete => "训练关卡完成",
        TrainingPanel.Error => failureTitle,
        _ => ""
    };

    private string PanelBody => panel switch {
        TrainingPanel.DemoComplete => "接下来由你完成动作，系统会在关键输入窗口自动放慢时间。",
        TrainingPanel.Failure => failureBody,
        TrainingPanel.AssistedComplete => "很好。最后进入自由练习；正确动作不会打断游戏。",
        TrainingPanel.LessonComplete => "动作与 Fuzz 可行窗口匹配。你可以继续探索或再练一次。",
        TrainingPanel.LevelComplete => "所有要求的练习点都已完成。",
        TrainingPanel.Error => failureBody,
        _ => ""
    };

    private Color PanelColor => panel switch {
        TrainingPanel.Failure or TrainingPanel.Error => Pink,
        TrainingPanel.LessonComplete or TrainingPanel.LevelComplete => Green,
        TrainingPanel.AssistedComplete => Yellow,
        _ => Cyan
    };

    private string[] PanelButtons => panel switch {
        TrainingPanel.DemoComplete => ["进入辅助实操", "退出练习"],
        TrainingPanel.Failure => ["重试本阶段", "退出练习"],
        TrainingPanel.AssistedComplete => ["进入自由练习", "再练辅助模式", "退出练习"],
        TrainingPanel.LessonComplete => ["继续游戏", "再练一次", "退出练习"],
        TrainingPanel.LevelComplete => ["返回项目选择", "继续自由练习"],
        TrainingPanel.Error => ["关闭"],
        _ => []
    };

    private IEnumerable<int> CandidateFrames(int inputIndex) => candidates.SelectMany(candidate =>
        candidate.VerifiedInputs.Where(input => input.InputIndex == inputIndex).Select(input => input.Frame)
    );

    private static List<TrainingFuzzInput> VerifiedSequence(TrainingLessonDefinition selected) => selected.Inputs
        .Where(input => input.Verify)
        .SkipWhile(input => !string.Equals(input.Id, selected.EntryInputId, StringComparison.Ordinal))
        .ToList();

    private int TimelineInputIndex(int currentFrame) {
        if (lesson is null || verifiedSequence.Count == 0) return -1;
        if (guidedMode && stage == TrainingLessonStage.Demo && fuzz is not null) {
            return fuzz.Best.VerifiedInputs
                .Where(input => input.Frame >= currentFrame && verifiedSequence.Any(expected => expected.Index == input.InputIndex))
                .OrderBy(input => input.Frame)
                .Select(input => input.InputIndex)
                .DefaultIfEmpty(verifiedSequence[^1].Index)
                .First();
        }
        return verifiedSequence[Math.Clamp(nextVerified, 0, verifiedSequence.Count - 1)].Index;
    }

    private int? TimelineTargetFrame(int inputIndex) => candidates
        .SelectMany(candidate => candidate.VerifiedInputs.Where(input => input.InputIndex == inputIndex))
        .Select(input => (int?) input.Frame)
        .FirstOrDefault()
        ?? fuzz?.Best.VerifiedInputs.FirstOrDefault(input => input.InputIndex == inputIndex)?.Frame;

    private void RenderObjectiveCurves(
        int inputIndex,
        int viewportStart,
        int viewportEnd,
        float left,
        float right,
        float top,
        float bottom
    ) {
        if (lesson is null || fuzz is null || inputIndex < 0) return;
        Color[] colors = [new Color(101, 217, 255), new Color(255, 116, 155), new Color(255, 211, 122)];
        IEnumerable<TrainingFuzzCandidate> source = candidates
            .Concat(fuzz.Evaluations.Where(MatchesActualPrefix));
        int count = Math.Min(3, lesson.ObjectiveExpressions.Count);
        for (int objectiveIndex = 0; objectiveIndex < count; objectiveIndex++) {
            var points = source
                .Where(candidate => candidate.ObjectiveValues.Count > objectiveIndex)
                .Select(candidate => new {
                    Input = candidate.VerifiedInputs.FirstOrDefault(input => input.InputIndex == inputIndex),
                    Value = candidate.ObjectiveValues[objectiveIndex],
                    candidate.Successful
                })
                .Where(point => point.Input is not null
                    && point.Input.Frame >= viewportStart
                    && point.Input.Frame <= viewportEnd
                    && double.IsFinite(point.Value))
                .GroupBy(point => point.Input!.Frame)
                .Select(group => group.OrderByDescending(point => point.Successful).First())
                .OrderBy(point => point.Input!.Frame)
                .ToArray();
            if (points.Length == 0) continue;
            double minimum = points.Min(point => point.Value);
            double maximum = points.Max(point => point.Value);
            double range = maximum - minimum;
            Vector2? previous = null;
            foreach (var point in points) {
                float x = MathHelper.Lerp(left, right, (point.Input!.Frame - viewportStart) / (float) Math.Max(1, viewportEnd - viewportStart));
                float y = range <= 0.000001
                    ? (top + bottom) * 0.5f
                    : MathHelper.Lerp(bottom, top, (float) ((point.Value - minimum) / range));
                Vector2 current = new(x, y);
                if (previous is Vector2 before) Draw.Line(before, current, colors[objectiveIndex] * 0.8f, 2f);
                Draw.Circle(current, point.Successful ? 3f : 2f, point.Successful ? colors[objectiveIndex] : Pink * 0.65f, 8);
                previous = current;
            }
        }
    }

    private bool MatchesActualPrefix(TrainingFuzzCandidate candidate) {
        for (int index = 0; index < actualInputs.Count; index++) {
            if (index >= verifiedSequence.Count) return false;
            (int frame, IReadOnlyList<string> keys) = actualInputs[index];
            TrainingVerifiedInput? expected = candidate.VerifiedInputs.FirstOrDefault(input =>
                input.InputIndex == verifiedSequence[index].Index);
            if (expected is null || expected.Frame != frame || !SameKeys(expected.Keys, keys)) return false;
        }
        return true;
    }

    private string ObjectiveSummary(int inputIndex) {
        if (lesson is null || fuzz is null || lesson.ObjectiveExpressions.Count == 0) return "";
        int objectiveIndex = Math.Min(lesson.ObjectiveExpressions.Count, fuzz.Best.ObjectiveValues.Count) - 1;
        if (objectiveIndex < 0) return "";
        string name = ObjectiveName(lesson.ObjectiveExpressions[objectiveIndex]);
        double value = fuzz.Best.ObjectiveValues[objectiveIndex];
        return double.IsFinite(value) ? $"{name}最佳 {value:0.##}" : name;
    }

    private static string ObjectiveName(string expression) => expression switch {
        "after.speed.x" or "final.speed.x" => "水平速度",
        "after.speed.y" or "final.speed.y" => "垂直速度",
        "after.dashes" => "冲刺次数",
        "after.stamina" => "体力",
        "after.pos.x" => "X 坐标",
        "after.pos.y" => "Y 坐标",
        _ => "目标"
    };

    private void UpdateHudPlacement() {
        Player? player = level.Tracker.GetEntity<Player>();
        if (player is null) return;
        float playerScreenX = (player.Center.X - level.Camera.X) * 6f;
        if (hudOnRight) {
            if (playerScreenX > 1110f) hudOnRight = false;
        } else if (playerScreenX < 810f) {
            hudOnRight = true;
        }
        float target = hudOnRight ? UiWidth - 80f - 620f : 80f;
        hudX = Calc.Approach(hudX, target, Engine.RawDeltaTime * 2100f);
    }

    private static bool AnyActionHeld(FrameButtons buttons) => buttons.Jump
        || buttons.Dash
        || buttons.CrouchDash
        || buttons.Grab
        || buttons.Talk;

    internal void BeforePlayerUpdate(Player player) {
        if (!guidedMode || stage != TrainingLessonStage.Demo || panel != TrainingPanel.None || lesson is null || fuzz is null) return;
        demoPreviousButtons = demoFrame <= 0 ? default : DemoButtonsAt(demoFrame - 1);
        demoButtons = demoPaused ? default : DemoButtonsAt(demoFrame);
        Input.MoveX.Value = demoButtons.Left == demoButtons.Right ? 0 : demoButtons.Right ? 1 : -1;
        Input.MoveY.Value = demoButtons.Up == demoButtons.Down ? 0 : demoButtons.Down ? 1 : -1;
    }

    internal void AfterPlayerUpdate(Player player) {
        if (!guidedMode || stage != TrainingLessonStage.Demo || panel != TrainingPanel.None || demoPaused || lesson is null || fuzz is null) return;
        int executedFrame = demoFrame;
        demoFrame++;
        IReadOnlyList<TrainingVerifiedInput> steps = DemoSteps;
        while (demoStepIndex < steps.Count && executedFrame >= steps[demoStepIndex].Frame) demoStepIndex++;
        bool reachedReplayTarget = !demoReplaying || demoStepIndex >= demoReplayTargetStep;
        if (demoStepIndex < steps.Count && demoFrame >= steps[demoStepIndex].Frame && reachedReplayTarget) {
            demoReplaying = false;
            demoPaused = true;
            timeRateModifier.Multiplier = 0f;
            return;
        }
        if (demoStepIndex >= steps.Count && demoFrame > DemoEndFrame()) {
            RestoreDemoInput();
            CaptureResultStats(true);
            ShowSuccessToast("演示完成", "停留一秒查看角色落点，随后进入辅助实操。");
            pendingStage = TrainingLessonStage.Assisted;
            stageTransitionTimer = 1f;
            timeRateModifier.Multiplier = 0f;
        }
    }

    internal bool TryGetDemoAim(Facings facing, out Vector2 aim) {
        if (!guidedMode || stage != TrainingLessonStage.Demo || panel != TrainingPanel.None) {
            aim = default;
            return false;
        }
        aim = new Vector2(
            demoButtons.Right ? 1f : demoButtons.Left ? -1f : 0f,
            demoButtons.Down ? 1f : demoButtons.Up ? -1f : 0f
        );
        if (aim == Vector2.Zero) aim = Vector2.UnitX * (float) facing;
        else aim = aim.SafeNormalize();
        return true;
    }

    private IReadOnlyList<TrainingVerifiedInput> DemoSteps => fuzz?.Best.VerifiedInputs
        .Where(input => verifiedSequence.Any(expected => expected.Index == input.InputIndex))
        .OrderBy(input => input.Frame)
        .ThenBy(input => input.InputIndex)
        .ToArray() ?? [];

    private int DemoEndFrame() {
        if (lesson is null || fuzz is null) return 36;
        int last = 0;
        foreach (TrainingFuzzInput input in lesson.Inputs) {
            int? at = ResolveDemoFrame(input);
            if (at is null) continue;
            int held = ResolveHeldFrames(input);
            last = Math.Max(last, at.Value + Math.Max(0, held - 1));
        }
        return last + 36;
    }

    private FrameButtons DemoButtonsAt(int frame) {
        if (lesson is null || fuzz is null || frame < 0) return default;
        HashSet<string> keys = new(StringComparer.Ordinal);
        foreach (TrainingFuzzInput input in lesson.Inputs) {
            int? at = ResolveDemoFrame(input);
            if (at is null) continue;
            int held = ResolveHeldFrames(input);
            bool active = held == int.MaxValue ? frame >= at : frame >= at && frame < at + held;
            if (active) keys.UnionWith(input.Keys);
        }
        return new FrameButtons(
            keys.Contains("up"),
            keys.Contains("down"),
            keys.Contains("left"),
            keys.Contains("right"),
            keys.Contains("jump"),
            keys.Contains("dash"),
            keys.Contains("crouch_dash"),
            keys.Contains("grab"),
            keys.Contains("talk")
        );
    }

    private int? ResolveDemoFrame(TrainingFuzzInput input) {
        if (fuzz is null) return null;
        if (input.At.ValueKind == JsonValueKind.Number) return input.At.GetInt32();
        if (input.At.ValueKind == JsonValueKind.String
            && fuzz.Best.Bindings.TryGetValue(input.At.GetString() ?? "", out int bound)) return bound;
        return null;
    }

    private int ResolveHeldFrames(TrainingFuzzInput input) {
        if (fuzz is null || input.HeldTime.ValueKind is JsonValueKind.Undefined or JsonValueKind.Null) return 1;
        if (input.HeldTime.ValueKind == JsonValueKind.Number) return Math.Max(1, input.HeldTime.GetInt32());
        if (input.HeldTime.ValueKind == JsonValueKind.String) {
            string value = input.HeldTime.GetString() ?? "";
            if (value == "hold::inf") return int.MaxValue;
            if (fuzz.Best.Bindings.TryGetValue(value, out int bound)) return Math.Max(1, bound);
        }
        return 1;
    }

    private void InstallDemoInput() {
        if (originalButtonNodes.Count > 0) return;
        ReplaceDemoNodes(Input.Jump, "jump");
        ReplaceDemoNodes(Input.Dash, "dash");
        ReplaceDemoNodes(Input.CrouchDash, "crouch_dash");
        ReplaceDemoNodes(Input.Grab, "grab");
        ReplaceDemoNodes(Input.Talk, "talk");
    }

    private void ReplaceDemoNodes(VirtualButton button, string key) {
        originalButtonNodes[button] = [.. button.Nodes];
        button.Nodes.Clear();
        button.Nodes.Add(new DemoButtonNode(
            () => demoButtons.Get(key),
            () => demoButtons.Get(key) && !demoPreviousButtons.Get(key),
            () => !demoButtons.Get(key) && demoPreviousButtons.Get(key)
        ) { Bufferable = false });
    }

    private void RestoreDemoInput() {
        foreach ((VirtualButton button, List<VirtualButton.Node> nodes) in originalButtonNodes) {
            button.Nodes.Clear();
            button.Nodes.AddRange(nodes);
        }
        originalButtonNodes.Clear();
        demoButtons = default;
        demoPreviousButtons = default;
    }

    private void ApplyInitialSnapshot() {
        if (lesson is null) return;
        Player? player = level.Tracker.GetEntity<Player>();
        if (player is null) return;
        JsonElement snapshot = lesson.InitialSnapshot;
        if (snapshot.TryGetProperty("pos", out JsonElement pos)) {
            player.Position = new Vector2(pos.GetProperty("x").GetSingle(), pos.GetProperty("y").GetSingle());
        }
        if (snapshot.TryGetProperty("speed", out JsonElement speed)) {
            player.Speed = new Vector2(speed.GetProperty("x").GetSingle(), speed.GetProperty("y").GetSingle());
        }
        if (snapshot.TryGetProperty("facing", out JsonElement facing)) player.Facing = facing.GetBoolean() ? Facings.Right : Facings.Left;
        if (snapshot.TryGetProperty("dashes", out JsonElement dashes)) player.Dashes = dashes.GetInt32();
        if (snapshot.TryGetProperty("stamina", out JsonElement stamina)) player.Stamina = stamina.GetSingle();
        if (snapshot.TryGetProperty("ducking", out JsonElement ducking)) player.Ducking = ducking.GetBoolean();
        if (snapshot.TryGetProperty("state", out JsonElement state)) player.StateMachine.State = PlayerStateIndex(state.GetString());
        level.Camera.Position = player.CameraTarget;
    }

    private static int PlayerStateIndex(string? state) => state switch {
        "Climb" => Player.StClimb,
        "Dash" => Player.StDash,
        "Swim" => Player.StSwim,
        "Boost" => Player.StBoost,
        "RedDash" => Player.StRedDash,
        "HitSquash" => Player.StHitSquash,
        "Launch" => Player.StLaunch,
        "Pickup" => Player.StPickup,
        "DreamDash" => Player.StDreamDash,
        "SummitLaunch" => Player.StSummitLaunch,
        "Dummy" => Player.StDummy,
        "StarFly" => Player.StStarFly,
        _ => Player.StNormal
    };

    private sealed class DemoButtonNode(Func<bool> check, Func<bool> pressed, Func<bool> released) : VirtualButton.Node {
        public override bool Check => check();
        public override bool Pressed => pressed();
        public override bool Released => released();
    }

    private static IEnumerable<(int From, int To)> Windows(IEnumerable<int> source) {
        int? from = null;
        int previous = 0;
        foreach (int frame in source.Distinct().Order()) {
            if (from is null) from = frame;
            else if (frame > previous + 1) {
                yield return (from.Value, previous);
                from = frame;
            }
            previous = frame;
        }
        if (from is not null) yield return (from.Value, previous);
    }

    private static string CompressFrames(IEnumerable<int> frames) => string.Join("、", Windows(frames).Select(window =>
        window.From == window.To ? $"F{window.From}" : $"F{window.From}–F{window.To}"
    ));

    private static FrameButtons CaptureButtons() => new(
        Up: Input.MoveY.Value < 0,
        Down: Input.MoveY.Value > 0,
        Left: Input.MoveX.Value < 0,
        Right: Input.MoveX.Value > 0,
        Jump: Input.Jump.Check,
        Dash: Input.Dash.Check,
        CrouchDash: Input.CrouchDash.Check,
        Grab: Input.Grab.Check,
        Talk: Input.Talk.Check
    );

    private static bool ExpectedTriggered(FrameButtons current, FrameButtons previous, IReadOnlyList<string> expected) {
        return expected.Count > 0
            && expected.All(current.Get)
            && expected.Any(key => !previous.Get(key));
    }

    private static bool AnyActionTriggered(FrameButtons current, FrameButtons previous) {
        string[] actions = ["jump", "dash", "crouch_dash", "grab", "talk"];
        return actions.Any(key => current.Get(key) && !previous.Get(key));
    }

    private static IReadOnlyList<string> VerificationKeys(FrameButtons current, FrameButtons previous, IReadOnlyList<string> expected) {
        string[] directions = ["up", "down", "left", "right"];
        string[] actions = ["jump", "dash", "crouch_dash", "grab", "talk"];
        IEnumerable<string> pressed = actions.Where(key => current.Get(key) && !previous.Get(key));
        return expected.Any(directions.Contains)
            ? directions.Where(current.Get).Concat(pressed).ToArray()
            : pressed.ToArray();
    }

    private static bool SameKeys(IEnumerable<string> left, IEnumerable<string> right) =>
        left.Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal)
            .SequenceEqual(right.Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal), StringComparer.Ordinal);

    private static JsonElement CaptureSnapshot(Player player) => JsonSerializer.SerializeToElement(new {
        pos = new { x = player.Position.X, y = player.Position.Y },
        speed = new { x = player.Speed.X, y = player.Speed.Y },
        state = PlayerStateName(player.StateMachine.State),
        facing = player.Facing == Facings.Right,
        dashes = player.Dashes,
        stamina = player.Stamina,
        on_ground = player.OnGround(),
        player_on_ground = player.OnGround(),
        player_on_ground_initialized = true,
        ducking = player.Ducking,
        dead = player.Dead,
        dash_dir = new { x = player.DashDir.X, y = player.DashDir.Y },
        last_aim = new { x = Input.GetAimVector(player.Facing).X, y = Input.GetAimVector(player.Facing).Y }
    });

    private static string PlayerStateName(int state) => state switch {
        Player.StNormal => "Normal",
        Player.StClimb => "Climb",
        Player.StDash => "Dash",
        Player.StSwim => "Swim",
        Player.StBoost => "Boost",
        Player.StRedDash => "RedDash",
        Player.StHitSquash => "HitSquash",
        Player.StLaunch => "Launch",
        Player.StPickup => "Pickup",
        Player.StDreamDash => "DreamDash",
        Player.StSummitLaunch => "SummitLaunch",
        Player.StDummy => "Dummy",
        Player.StStarFly => "StarFly",
        _ => "Normal"
    };

    private static string HumanKeys(IEnumerable<string> keys) => string.Join(" + ", keys.Select(key => key switch {
        "jump" => "跳跃",
        "dash" => "冲刺",
        "crouch_dash" => "蹲冲",
        "grab" => "抓取",
        "left" => "左",
        "right" => "右",
        "up" => "上",
        "down" => "下",
        _ => key
    }));

    private string CompletionFlag(string lessonId) => $"celeste_gym_completed_{project.Id}_{lessonId}";

    private static Vector2 MousePositionInUi() {
        Rectangle viewport = Engine.Viewport.Bounds;
        Vector2 mouse = MInput.Mouse.Position - viewport.Location.ToVector2();
        return viewport.Width <= 0 || viewport.Height <= 0
            ? mouse
            : new Vector2(mouse.X * UiWidth / viewport.Width, mouse.Y * UiHeight / viewport.Height);
    }

    private static Point MousePointInUi() {
        Vector2 mouse = MousePositionInUi();
        return new Point((int) mouse.X, (int) mouse.Y);
    }

    private static Rectangle LaunchBoundsAt(int panelX) => new(panelX + 105, 242, 410, 44);

    private Rectangle DemoButtonBounds(int index) => new((int) hudX + 85 + index * 230, 196, 200, 40);

    private Rectangle PanelButtonBounds(int index, int count) {
        int width = count switch { 1 => 330, 2 => 330, _ => 300 };
        int gap = 28;
        int total = count * width + (count - 1) * gap;
        if (panel == TrainingPanel.Failure) {
            width = 280;
            total = count * width + (count - 1) * gap;
            return new Rectangle((1920 - total) / 2 + index * (width + gap), 668, width, 58);
        }
        bool hasAttemptResult = panel is TrainingPanel.AssistedComplete or TrainingPanel.LessonComplete;
        return new Rectangle((1920 - total) / 2 + index * (width + gap), hasAttemptResult ? 824 : 620, width, 72);
    }

    private static void RenderMouseCursor(Vector2 mouse) {
        Draw.Line(mouse + new Vector2(4, 5), mouse + new Vector2(4, 39), Color.Black, 9);
        Draw.Line(mouse + new Vector2(4, 5), mouse + new Vector2(27, 29), Color.Black, 9);
        Draw.Line(mouse, mouse + new Vector2(0, 34), Color.White, 5);
        Draw.Line(mouse, mouse + new Vector2(23, 24), Color.White, 5);
        Draw.Line(mouse + new Vector2(0, 34), mouse + new Vector2(10, 24), Color.White, 5);
    }

    private readonly record struct FrameButtons(
        bool Up,
        bool Down,
        bool Left,
        bool Right,
        bool Jump,
        bool Dash,
        bool CrouchDash,
        bool Grab,
        bool Talk
    ) {
        public bool Get(string key) => key switch {
            "up" => Up,
            "down" => Down,
            "left" => Left,
            "right" => Right,
            "jump" => Jump,
            "dash" => Dash,
            "crouch_dash" => CrouchDash,
            "grab" => Grab,
            "talk" => Talk,
            _ => false
        };
    }
}
