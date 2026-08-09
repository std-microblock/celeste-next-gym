using System.Reflection;
using Microsoft.Xna.Framework;
using Mono.Cecil.Cil;
using MonoMod.Cil;
using MonoMod.RuntimeDetour;

namespace Celeste.Mod.CelesteGymCollector;

/// <summary>
/// Expands one repository-owned FNA fixed-step tick into an explicitly requested
/// number of normal virtual Update calls. FNA itself advances GameTime by the
/// configured fixed timestep for every call, so Celeste, Everest hooks, Mod
/// entities, and coroutines all execute through the real game loop.
/// </summary>
internal sealed class GymFastLoopPatch : IDisposable {
    private static readonly FieldInfo AccumulatedElapsedTime = RequireField("accumulatedElapsedTime");
    private static readonly FieldInfo UpdateFrameLag = RequireField("updateFrameLag");
    private static readonly FieldInfo CurrentGameTime = RequireField("gameTime");
    private static readonly PropertyInfo ElapsedGameTime = RequireGameTimeProperty("ElapsedGameTime");
    private static readonly PropertyInfo IsRunningSlowly = RequireGameTimeProperty("IsRunningSlowly");

    private readonly Func<int> selectFrameCount;
    private readonly Func<bool> isStepActive;
    private readonly ILHook tickHook;
    private bool acceleratedTickActive;

    public GymFastLoopPatch(Func<int> selectFrameCount, Func<bool> isStepActive) {
        this.selectFrameCount = selectFrameCount;
        this.isStepActive = isStepActive;
        MethodInfo tick = typeof(Game).GetMethod(
            nameof(Game.Tick),
            BindingFlags.Instance | BindingFlags.Public
        ) ?? throw new MissingMethodException(typeof(Game).FullName, nameof(Game.Tick));
        tickHook = new ILHook(tick, PatchTick);
    }

    public void Dispose() => tickHook.Dispose();

    private void PatchTick(ILContext context) {
        ILCursor cursor = new(context);

        // Insert after FNA's wall-clock MaxElapsedTime clamp but before the
        // fixed-step branch. The injected value replaces wall-clock debt with
        // exactly N fixed physics frames for this opt-in gym request.
        if (!cursor.TryGotoNext(
                MoveType.After,
                instruction => instruction.MatchLdsfld<Game>("MaxElapsedTime"),
                instruction => instruction.MatchStfld<Game>("accumulatedElapsedTime")
            )) {
            throw new InvalidOperationException(
                "CelesteGymCollector could not locate FNA Tick elapsed-time clamp"
            );
        }
        cursor.Emit(OpCodes.Ldarg_0);
        cursor.EmitDelegate<Action<Game>>(PrepareTick);

        // This point runs after every fixed Update. If the episode terminated
        // before consuming the requested batch, clear the remaining synthetic
        // time so FNA cannot advance the scene after the returned terminal state.
        if (!cursor.TryGotoNext(
                MoveType.After,
                instruction => instruction.MatchLdarg(0),
                instruction => instruction.MatchLdarg(0),
                instruction => instruction.MatchLdfld<Game>("gameTime"),
                instruction => instruction.MatchCallvirt<Game>("Update")
            )) {
            throw new InvalidOperationException(
                "CelesteGymCollector could not locate FNA Tick fixed Update call"
            );
        }
        cursor.Emit(OpCodes.Ldarg_0);
        cursor.EmitDelegate<Action<Game>>(AfterFixedUpdate);

        // FNA normally interprets multiple updates in one tick as rendering
        // lag. The gym batch is intentional, so do not leak a thousands-frame
        // IsRunningSlowly debt into later game logic.
        if (!cursor.TryGotoNext(
                MoveType.Before,
                instruction => instruction.MatchLdarg(0),
                instruction => instruction.MatchLdfld<Game>("suppressDraw")
            )) {
            throw new InvalidOperationException(
                "CelesteGymCollector could not locate FNA Tick draw boundary"
            );
        }
        cursor.Emit(OpCodes.Ldarg_0);
        cursor.EmitDelegate<Action<Game>>(FinishTick);
    }

    private void PrepareTick(Game game) {
        acceleratedTickActive = false;
        int frames = selectFrameCount();
        if (frames is < 1 or > GymFastLoopPolicy.MaximumBatchFrames) return;
        TimeSpan fixedStep = game.TargetElapsedTime;
        if (fixedStep <= TimeSpan.Zero) return;

        AccumulatedElapsedTime.SetValue(
            game,
            TimeSpan.FromTicks(checked(fixedStep.Ticks * frames))
        );
        acceleratedTickActive = true;
    }

    private void AfterFixedUpdate(Game game) {
        if (acceleratedTickActive && !isStepActive()) {
            AccumulatedElapsedTime.SetValue(game, TimeSpan.Zero);
        }
    }

    private void FinishTick(Game game) {
        if (!acceleratedTickActive) return;
        UpdateFrameLag.SetValue(game, 0);
        if (CurrentGameTime.GetValue(game) is GameTime gameTime) {
            IsRunningSlowly.SetValue(gameTime, false);
            ElapsedGameTime.SetValue(gameTime, game.TargetElapsedTime);
        }
        acceleratedTickActive = false;
    }

    private static FieldInfo RequireField(string name) => typeof(Game).GetField(
        name,
        BindingFlags.Instance | BindingFlags.NonPublic
    ) ?? throw new MissingFieldException(typeof(Game).FullName, name);

    private static PropertyInfo RequireGameTimeProperty(string name) => typeof(GameTime).GetProperty(
        name,
        BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic
    ) ?? throw new MissingMemberException(typeof(GameTime).FullName, name);
}
