using System.Collections;
using System.Reflection;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Input;
using Monocle;
using MonoMod.RuntimeDetour;

namespace Celeste.Mod.MicroblocksQolUtils;

public sealed class MaterialChapterSelect : Oui {
    private const float ScreenWidth = 1920f;
    private const float ScreenHeight = 1080f;
    private const float SidebarX = 42f;
    private const float SidebarY = 146f;
    private const float SidebarWidth = 310f;
    private const float SidebarItemHeight = 62f;
    private const int SidebarVisibleItems = 13;
    private const float ContentX = 394f;
    private const float ContentY = 166f;
    private const float CardWidth = 466f;
    private const float CardHeight = 230f;
    private const float CardGapX = 26f;
    private const float CardGapY = 24f;
    private const int Columns = 3;
    private const int VisibleRows = 3;

    private static Hook? gotoRoutineHook;
    private static bool hookFailed;
    private static bool replaceNextChapterSelect;
    private static bool materialSessionActive;

    private readonly List<ChapterEntry> allEntries = [];
    private readonly List<ChapterEntry> entries = [];
    private readonly List<LevelSetEntry> levelSets = [];
    private int selectedIndex;
    private int selectedLevelSet;
    private int cardScrollRow;
    private int levelSetScroll;
    private float ease;
    private bool display;
    private Color paletteSeed = new(126, 99, 184);

    internal bool SuppressNormalRender { get; set; }

    internal static MaterialChapterSelect? ActivePage {
        get {
            if (Engine.Scene is not Overworld overworld) return null;
            MaterialChapterSelect? page = overworld.Current as MaterialChapterSelect
                ?? overworld.Next as MaterialChapterSelect;
            return page is { Visible: true } ? page : null;
        }
    }

    public static void Load() {
        if (gotoRoutineHook is not null || hookFailed) return;
        try {
            On.Celeste.OuiFileSelectSlot.OnContinueSelected += FileSelectContinue;
            MethodInfo method = typeof(Overworld).GetMethod(
                "GotoRoutine",
                BindingFlags.Instance | BindingFlags.NonPublic
            ) ?? throw new MissingMethodException(typeof(Overworld).FullName, "GotoRoutine");
            gotoRoutineHook = new Hook(method, (GotoRoutineDetour)DetourGotoRoutine);
        } catch (Exception exception) {
            On.Celeste.OuiFileSelectSlot.OnContinueSelected -= FileSelectContinue;
            hookFailed = true;
            Logger.LogDetailed(exception, "MicroblocksQolUtils/MaterialChapterSelect");
        }
    }

    public static void Unload() {
        On.Celeste.OuiFileSelectSlot.OnContinueSelected -= FileSelectContinue;
        gotoRoutineHook?.Dispose();
        gotoRoutineHook = null;
        hookFailed = false;
        replaceNextChapterSelect = false;
        materialSessionActive = false;
    }

    public override bool IsStart(Overworld overworld, Overworld.StartMode start) => false;

    public override IEnumerator Enter(Oui from) {
        Visible = true;
        display = true;
        ease = 0f;
        Overworld.ShowInputUI = false;
        Overworld.Mountain.AllowUserRotation = false;
        Overworld.Maddy.Hide();
        RebuildEntries();
        paletteSeed = entries.Count == 0
            ? new Color(126, 99, 184)
            : entries[Math.Clamp(selectedIndex, 0, entries.Count - 1)].Area.TitleBaseColor;
        materialSessionActive = true;
        Audio.Play("event:/ui/world_map/icon/roll_right");
        yield return null;
    }

    public override IEnumerator Leave(Oui next) {
        display = false;
        float duration = 0.16f;
        for (float timer = 0f; timer < duration; timer += Engine.DeltaTime) yield return null;
        Visible = false;
        Overworld.ShowInputUI = true;
        Overworld.Mountain.AllowUserRotation = true;
    }

    public override void Update() {
        ease = Calc.Approach(ease, display ? 1f : 0f, Engine.DeltaTime * 7f);
        if (Focused && display) UpdateInput();
        base.Update();
    }

    public override void Render() {
        if (SuppressNormalRender || !MicroblocksQolUtilsModule.Settings.MaterialYouInterface) return;
        RenderMaterialContent(acrylicActive: false);
    }

    internal void RenderMaterialContent(bool acrylicActive) {
        if (!Visible || ease <= 0f) return;
        ChapterEntry? selected = entries.Count == 0 ? null : entries[Math.Clamp(selectedIndex, 0, entries.Count - 1)];
        MaterialPalette palette = MaterialPalette.FromSeed(paletteSeed);
        float eased = Ease.CubeOut(ease);
        Draw.Rect(0f, 0f, ScreenWidth, ScreenHeight, palette.Scrim * eased);

        float rise = (1f - eased) * 34f;
        MaterialUiKit.Surface(new MaterialRect(28f, 24f + rise, 1864f, 1030f), 42f,
            palette with { SurfaceHigh = palette.Surface * (acrylicActive ? 0.78f : 0.94f) }, eased);

        MaterialUiKit.Text(UiText("microblocks_qol_chapter_title", "选择章节"),
            new Vector2(70f, 58f + rise), Vector2.Zero, MaterialTextRole.Display,
            palette.OnSurface, eased);
        string subtitle = CollabUtils2Bridge.Available
            ? UiText("microblocks_qol_chapter_subtitle_collab", "Material You  ·  键盘 / 鼠标  ·  CollabUtils2")
            : UiText("microblocks_qol_chapter_subtitle", "Material You  ·  键盘 / 鼠标");
        MaterialUiKit.Text(subtitle, new Vector2(72f, 111f + rise), Vector2.Zero,
            MaterialTextRole.Body, palette.OnSurfaceVariant, eased);

        RenderLevelSets(palette, rise, eased);
        RenderCards(palette, rise, eased);
        RenderFooter(palette, selected, rise, eased);
        RenderMouseCursor(palette, eased);
    }

    private void UpdateInput() {
        if (Input.MenuCancel.Pressed || MInput.Keyboard.Pressed(Keys.Escape)) {
            Audio.Play("event:/ui/main/button_back");
            materialSessionActive = false;
            Overworld.Goto<OuiFileSelect>();
            return;
        }

        if (MInput.Keyboard.Pressed(Keys.Tab)) {
            int direction = MInput.Keyboard.Check(Keys.LeftShift, Keys.RightShift) ? -1 : 1;
            SelectLevelSet(selectedLevelSet + direction);
        }

        if (entries.Count > 0) {
            if (Input.MenuLeft.Pressed) MoveSelection(-1);
            else if (Input.MenuRight.Pressed) MoveSelection(1);
            else if (Input.MenuUp.Pressed) MoveSelection(-Columns);
            else if (Input.MenuDown.Pressed) MoveSelection(Columns);
            else if (Input.MenuConfirm.Pressed || MInput.Keyboard.Pressed(Keys.Enter)) ActivateSelected();
        }

        Vector2 mouse = MInput.Mouse.Position;
        bool inSidebar = MaterialUi.Contains(mouse, SidebarX, SidebarY, SidebarWidth, 850f);
        if (MInput.Mouse.WheelDelta != 0) {
            if (inSidebar) {
                levelSetScroll = Math.Clamp(
                    levelSetScroll - Math.Sign(MInput.Mouse.WheelDelta),
                    0,
                    Math.Max(0, levelSets.Count - SidebarVisibleItems)
                );
            } else {
                cardScrollRow = Math.Clamp(
                    cardScrollRow - Math.Sign(MInput.Mouse.WheelDelta),
                    0,
                    MaxCardScrollRow()
                );
            }
        }

        if (MInput.Mouse.WasMoved || MInput.Mouse.PressedLeftButton) {
            int sidebar = SidebarIndexAt(mouse);
            if (sidebar >= 0) {
                if (MInput.Mouse.PressedLeftButton) SelectLevelSet(sidebar);
                return;
            }
            int card = CardIndexAt(mouse);
            if (card >= 0) {
                if (selectedIndex != card) {
                    selectedIndex = card;
                    Audio.Play("event:/ui/world_map/icon/roll_right");
                }
                if (MInput.Mouse.PressedLeftButton) ActivateSelected();
            }
        }
    }

    private void RebuildEntries() {
        allEntries.Clear();
        levelSets.Clear();
        levelSets.Add(new LevelSetEntry("", UiText("microblocks_qol_chapter_all_maps", "全部地图")));
        bool showCollabMaps = MicroblocksQolUtilsModule.Settings.ChapterSelectShowCollabMaps;
        SaveData? save = SaveData.Instance;
        foreach (AreaData area in AreaData.Areas) {
            if (area.Mode is null || area.Mode.Length == 0 || area.Mode[0] is null) continue;
            string sid = area.SID ?? area.ID.ToString();
            bool collabMap = CollabUtils2Bridge.IsCollabMap(sid);
            bool collabGym = CollabUtils2Bridge.IsCollabGym(sid);
            if (!showCollabMaps && (collabMap || collabGym)) continue;
            if (save is not null
                && area.LevelSet == "Celeste"
                && area.ID > save.UnlockedAreas
                && !(save.AssistMode && area.ID <= save.MaxAssistArea)) continue;

            string levelSet = area.LevelSet ?? "Celeste";
            string title = CleanName(area.Name, sid);
            string levelSetTitle = levelSet == "Celeste" ? "Celeste" : Dialog.CleanLevelSet(levelSet);
            string badge = CollabUtils2Bridge.IsCollabLobby(sid) ? "LOBBY"
                : collabGym ? "GYM"
                : collabMap ? "COLLAB"
                : levelSet == "Celeste" ? UiText("microblocks_qol_chapter_official", "官方") : "MOD";
            allEntries.Add(new ChapterEntry(area, sid, levelSet, title, levelSetTitle, badge));
            if (levelSets.All(item => !string.Equals(item.Id, levelSet, StringComparison.Ordinal)))
                levelSets.Add(new LevelSetEntry(levelSet, levelSetTitle));
        }

        string currentLevelSet = save?.LevelSet ?? "";
        selectedLevelSet = Math.Max(0, levelSets.FindIndex(item => item.Id == currentLevelSet));
        FilterEntries(keepArea: true);
    }

    private void FilterEntries(bool keepArea) {
        string? previousSid = keepArea && entries.Count > 0 && selectedIndex < entries.Count
            ? entries[selectedIndex].Sid
            : SaveData.Instance?.LastArea.SID;
        entries.Clear();
        string levelSet = levelSets.Count == 0 ? "" : levelSets[selectedLevelSet].Id;
        entries.AddRange(levelSet.Length == 0
            ? allEntries
            : allEntries.Where(entry => entry.LevelSet == levelSet));
        selectedIndex = Math.Max(0, entries.FindIndex(entry => entry.Sid == previousSid));
        if (entries.Count == 0) selectedIndex = 0;
        cardScrollRow = 0;
        EnsureSelectionVisible();
        EnsureLevelSetVisible();
    }

    private void SelectLevelSet(int index) {
        if (levelSets.Count == 0) return;
        index = (index % levelSets.Count + levelSets.Count) % levelSets.Count;
        if (index == selectedLevelSet) return;
        selectedLevelSet = index;
        Audio.Play("event:/ui/world_map/icon/roll_right");
        FilterEntries(keepArea: false);
    }

    private void MoveSelection(int delta) {
        if (entries.Count == 0) return;
        int next = Math.Clamp(selectedIndex + delta, 0, entries.Count - 1);
        if (next == selectedIndex) return;
        selectedIndex = next;
        Audio.Play(delta < 0 ? "event:/ui/world_map/icon/roll_left" : "event:/ui/world_map/icon/roll_right");
        EnsureSelectionVisible();
    }

    private void EnsureSelectionVisible() {
        int row = selectedIndex / Columns;
        if (row < cardScrollRow) cardScrollRow = row;
        if (row >= cardScrollRow + VisibleRows) cardScrollRow = row - VisibleRows + 1;
        cardScrollRow = Math.Clamp(cardScrollRow, 0, MaxCardScrollRow());
    }

    private void EnsureLevelSetVisible() {
        if (selectedLevelSet < levelSetScroll) levelSetScroll = selectedLevelSet;
        if (selectedLevelSet >= levelSetScroll + SidebarVisibleItems)
            levelSetScroll = selectedLevelSet - SidebarVisibleItems + 1;
        levelSetScroll = Math.Clamp(levelSetScroll, 0, Math.Max(0, levelSets.Count - SidebarVisibleItems));
    }

    private void ActivateSelected() {
        SaveData? save = SaveData.Instance;
        if (entries.Count == 0 || save is null) {
            Audio.Play("event:/ui/main/button_invalid");
            return;
        }
        ChapterEntry entry = entries[selectedIndex];
        Audio.Play("event:/ui/world_map/icon/select");
        save.LastArea = entry.Area.ToKey();
        UserIO.SaveHandler(file: true, settings: false);
        Overworld.Goto<OuiChapterPanel>();
    }

    private void RenderLevelSets(MaterialPalette palette, float rise, float alpha) {
        MaterialUiKit.Surface(new MaterialRect(SidebarX, SidebarY + rise, SidebarWidth, 850f),
            28f, palette with { SurfaceHigh = palette.SurfaceHigh * 0.82f }, alpha);
        MaterialUiKit.Text(UiText("microblocks_qol_chapter_level_sets", "地图集"),
            new Vector2(SidebarX + 26f, SidebarY + 20f + rise), Vector2.Zero,
            MaterialTextRole.Section, palette.OnSurface, alpha);
        for (int visible = 0; visible < SidebarVisibleItems; visible++) {
            int index = levelSetScroll + visible;
            if (index >= levelSets.Count) break;
            float y = SidebarY + 70f + visible * SidebarItemHeight + rise;
            bool selected = index == selectedLevelSet;
            MaterialUiKit.NavigationPill(new MaterialRect(SidebarX + 14f, y, SidebarWidth - 28f, 52f),
                palette, selected, alpha);
            SystemTtfFont.Draw(
                Trim(levelSets[index].Title, 20),
                new Vector2(SidebarX + 34f, y + 11f),
                Vector2.Zero,
                0.39f,
                (selected ? palette.OnPrimary : palette.OnSurfaceVariant) * alpha,
                weight: selected ? UiFontWeight.Bold : UiFontWeight.Regular
            );
        }
    }

    private void RenderCards(MaterialPalette palette, float rise, float alpha) {
        int first = cardScrollRow * Columns;
        int last = Math.Min(entries.Count, first + VisibleRows * Columns);
        for (int index = first; index < last; index++) {
            int visible = index - first;
            int column = visible % Columns;
            int row = visible / Columns;
            float x = ContentX + column * (CardWidth + CardGapX);
            float y = ContentY + row * (CardHeight + CardGapY) + rise;
            bool selected = index == selectedIndex;
            Color surface = selected ? palette.SurfaceHighest : palette.SurfaceHigh;
            MaterialUiKit.Card(new MaterialRect(x, y, CardWidth, CardHeight),
                palette with { SurfaceHigh = surface * (selected ? 0.98f : 0.85f) }, selected, alpha);
            if (selected)
                MaterialUi.RoundedRect(x + 18f, y + 18f, 8f, CardHeight - 36f, 4f,
                    palette.Primary * alpha);
            RenderCard(entries[index], x, y, selected, palette, alpha);
        }
        if (entries.Count == 0) {
            SystemTtfFont.Draw(UiText("microblocks_qol_chapter_empty", "这个地图集中没有可选章节"),
                new Vector2(1110f, 500f + rise),
                new Vector2(0.5f), 0.62f, palette.OnSurfaceVariant * alpha);
        }
    }

    private static void RenderCard(
        ChapterEntry entry,
        float x,
        float y,
        bool selected,
        MaterialPalette palette,
        float alpha
    ) {
        if (!string.IsNullOrWhiteSpace(entry.Area.Icon) && GFX.Gui.Has(entry.Area.Icon)) {
            MTexture icon = GFX.Gui[entry.Area.Icon];
            float scale = Math.Min(1f, 78f / Math.Max(icon.Width, icon.Height));
            icon.DrawCentered(new Vector2(x + 76f, y + 77f), Color.White * alpha, scale);
        } else {
            MaterialUi.RoundedRect(x + 38f, y + 39f, 76f, 76f, 24f, palette.Primary * 0.42f * alpha);
        }
        SystemTtfFont.Draw(Trim(entry.Title, 22), new Vector2(x + 132f, y + 33f),
            Vector2.Zero, 0.48f, palette.OnSurface * alpha, weight: UiFontWeight.Bold);
        SystemTtfFont.Draw(Trim(entry.LevelSetTitle, 27), new Vector2(x + 132f, y + 79f),
            Vector2.Zero, 0.31f, palette.OnSurfaceVariant * alpha);

        AreaStats? stats = SaveData.Instance?.GetAreaStatsFor(entry.Area.ToKey());
        AreaModeStats? mode = stats?.Modes is { Length: > 0 } ? stats.Modes[0] : null;
        string progress = mode is null
            ? UiText("microblocks_qol_chapter_never_entered", "尚未游玩")
            : $"{UiText(mode.Completed ? "microblocks_qol_chapter_cleared" : "microblocks_qol_chapter_uncleared", mode.Completed ? "已完成" : "进行中")}  ·  "
                + $"{mode.TotalStrawberries} {UiText("microblocks_qol_chapter_berries", "草莓")}  ·  "
                + $"{mode.Deaths} {UiText("microblocks_qol_chapter_deaths", "死亡")}";
        SystemTtfFont.Draw(progress, new Vector2(x + 38f, y + 145f), Vector2.Zero, 0.31f,
            palette.OnSurfaceVariant * alpha);

        MaterialUiKit.Chip(entry.Badge,
            new Vector2(x + CardWidth - 22f, y + CardHeight - 52f), palette, selected, alpha);
    }

    private static void RenderFooter(
        MaterialPalette palette,
        ChapterEntry? selected,
        float rise,
        float alpha
    ) {
        float y = 954f + rise;
        string detail = selected is null
            ? UiText("microblocks_qol_chapter_no_available", "没有可用章节")
            : selected.Sid;
        SystemTtfFont.Draw(Trim(detail, 72), new Vector2(72f, y), Vector2.Zero, 0.31f,
            palette.OnSurfaceVariant * alpha);
        SystemTtfFont.Draw(UiText("microblocks_qol_chapter_controls",
                "Enter / 左键：打开   Esc：返回   Tab：切换地图集   滚轮：滚动"),
            new Vector2(1844f, y), new Vector2(1f, 0f), 0.31f,
            palette.OnSurfaceVariant * alpha);
    }

    private static void RenderMouseCursor(MaterialPalette palette, float alpha) {
        if (!MInput.Mouse.WasMoved && !MInput.Mouse.CheckLeftButton) return;
        Vector2 mouse = MInput.Mouse.Position;
        MaterialUiKit.Cursor(mouse, palette, alpha);
    }

    private int SidebarIndexAt(Vector2 mouse) {
        if (!MaterialUi.Contains(mouse, SidebarX + 12f, SidebarY + 66f, SidebarWidth - 24f,
                SidebarVisibleItems * SidebarItemHeight)) return -1;
        int visible = (int)((mouse.Y - SidebarY - 66f) / SidebarItemHeight);
        int index = levelSetScroll + visible;
        return index >= 0 && index < levelSets.Count ? index : -1;
    }

    private int CardIndexAt(Vector2 mouse) {
        float localX = mouse.X - ContentX;
        float localY = mouse.Y - ContentY;
        if (localX < 0f || localY < 0f) return -1;
        int column = (int)(localX / (CardWidth + CardGapX));
        int row = (int)(localY / (CardHeight + CardGapY));
        if (column < 0 || column >= Columns || row < 0 || row >= VisibleRows) return -1;
        if (localX % (CardWidth + CardGapX) > CardWidth || localY % (CardHeight + CardGapY) > CardHeight)
            return -1;
        int index = (cardScrollRow + row) * Columns + column;
        return index < entries.Count ? index : -1;
    }

    private int MaxCardScrollRow() => Math.Max(0, (entries.Count + Columns - 1) / Columns - VisibleRows);

    private static string CleanName(string dialogKey, string fallback) {
        string value = Dialog.Clean(dialogKey ?? "");
        return string.IsNullOrWhiteSpace(value) || value == dialogKey ? fallback : value;
    }

    private static string UiText(string key, string fallback) {
        string value = Dialog.Clean(key);
        return string.IsNullOrWhiteSpace(value) || value == key ? fallback : value;
    }

    private static string Trim(string value, int maxCharacters) => value.Length <= maxCharacters
        ? value
        : value[..Math.Max(1, maxCharacters - 1)] + "…";

    private static IEnumerator DetourGotoRoutine(GotoRoutineOrig orig, Overworld self, Oui next) {
        if (next is OuiChapterSelect vanilla
            && MicroblocksQolUtilsModule.Settings.ReplaceChapterSelect
            && MicroblocksQolUtilsModule.Settings.MaterialYouInterface
            && (replaceNextChapterSelect || materialSessionActive && self.Current is OuiChapterPanel)
            && !IsAutoAdvancing(vanilla)
            && self.GetUI<MaterialChapterSelect>() is { } material) {
            next = material;
        }
        return orig(self, next);
    }

    private static void FileSelectContinue(
        On.Celeste.OuiFileSelectSlot.orig_OnContinueSelected orig,
        OuiFileSelectSlot self
    ) {
        replaceNextChapterSelect = true;
        try {
            orig(self);
        } finally {
            replaceNextChapterSelect = false;
        }
    }

    private static bool IsAutoAdvancing(OuiChapterSelect select) {
        try {
            return (bool?)typeof(OuiChapterSelect)
                .GetField("autoAdvancing", BindingFlags.Instance | BindingFlags.NonPublic)
                ?.GetValue(select) == true;
        } catch {
            return false;
        }
    }

    private delegate IEnumerator GotoRoutineOrig(Overworld self, Oui next);
    private delegate IEnumerator GotoRoutineDetour(GotoRoutineOrig orig, Overworld self, Oui next);

    private sealed record ChapterEntry(
        AreaData Area,
        string Sid,
        string LevelSet,
        string Title,
        string LevelSetTitle,
        string Badge
    );

    private sealed record LevelSetEntry(string Id, string Title);
}
