using System.Collections;
using System.Collections.Concurrent;
using System.Reflection;
using Microsoft.Xna.Framework;
using Monocle;

namespace Celeste.Mod.CelesteGymCollector;

internal sealed class GymEpisode(
    int areaId,
    string areaSid,
    string startRoom,
    int maxFrames,
    bool includeEntities
) {
    private readonly Dictionary<Entity, int> entityIds = new(ReferenceEqualityComparer.Instance);
    private int nextEntityId = 1;

    public string Id { get; } = Guid.NewGuid().ToString("N");
    public int AreaId { get; } = areaId;
    public string AreaSid { get; } = areaSid;
    public string StartRoom { get; } = startRoom;
    public int MaxFrames { get; } = maxFrames;
    public bool IncludeEntities { get; } = includeEntities;
    public int Frame { get; set; }
    public PlayerFrame? LastPlayer { get; set; }
    public bool Done { get; set; }
    public bool Terminated { get; set; }
    public bool Truncated { get; set; }
    public bool Success { get; set; }
    public string? TerminationReason { get; set; }

    public int EntityId(Entity entity) {
        if (entityIds.TryGetValue(entity, out int id)) return id;
        id = nextEntityId++;
        entityIds.Add(entity, id);
        return id;
    }
}

internal static class GymCapture {
    private const int MaxReflectedFields = 64;
    private const int MaxCollectionItems = 64;
    private static readonly ConcurrentDictionary<Type, FieldInfo[]> entityFields = new();

    public static GymObservation Capture(
        Level level,
        GymEpisode episode,
        PlayerFrame player,
        bool includeGeometry,
        bool terminated,
        bool truncated,
        bool success,
        string? terminationReason
    ) => new() {
        EpisodeId = episode.Id,
        EpisodeFrame = episode.Frame,
        AreaId = level.Session.Area.ID,
        AreaSid = level.Session.Area.SID,
        Room = level.Session.Level,
        Player = player,
        RoomGeometry = includeGeometry ? CaptureGeometry(level) : null,
        Entities = episode.IncludeEntities ? CaptureEntities(level, episode) : [],
        Terminated = terminated,
        Truncated = truncated,
        Success = success,
        TerminationReason = terminationReason
    };

    public static GymRoomGeometry CaptureGeometry(Level level) {
        Rectangle bounds = level.Bounds;
        Rectangle roomTiles = level.Session.LevelData.TileBounds;
        Rectangle mapTiles = level.Session.MapData.TileBounds;
        List<string> rows = new(roomTiles.Height);
        for (int y = 0; y < roomTiles.Height; y++) {
            char[] row = new char[roomTiles.Width];
            for (int x = 0; x < roomTiles.Width; x++) {
                int mapX = roomTiles.X + x - mapTiles.X;
                int mapY = roomTiles.Y + y - mapTiles.Y;
                row[x] = level.SolidsData.SafeCheck(mapX, mapY) == '0' ? '0' : '1';
            }
            rows.Add(new string(row));
        }
        return new GymRoomGeometry {
            Bounds = [bounds.X, bounds.Y, bounds.Width, bounds.Height],
            TileOrigin = [roomTiles.X, roomTiles.Y],
            Width = roomTiles.Width,
            Height = roomTiles.Height,
            Solids = rows
        };
    }

    private static List<GymEntityFrame> CaptureEntities(Level level, GymEpisode episode) {
        List<GymEntityFrame> result = [];
        foreach (Entity entity in level.Entities) {
            if (entity is Player) continue;
            Collider? collider = entity.Collider;
            result.Add(new GymEntityFrame {
                Id = episode.EntityId(entity),
                Type = entity.GetType().FullName ?? entity.GetType().Name,
                Position = [entity.Position.X, entity.Position.Y],
                Collider = collider is null ? null : SnapshotCapture.ColliderGeometry(collider),
                ColliderType = collider?.GetType().FullName ?? collider?.GetType().Name,
                Speed = ReadVector(entity, "Speed", "speed"),
                LiftSpeed = ReadVector(entity, "LiftSpeed", "liftSpeed"),
                Active = entity.Active,
                Visible = entity.Visible,
                Collidable = entity.Collidable,
                Depth = entity.Depth,
                Tag = entity.Tag,
                Fields = CaptureFields(entity)
            });
        }
        return result;
    }

    private static Dictionary<string, object?> CaptureFields(Entity entity) {
        Dictionary<string, object?> result = [];
        foreach (FieldInfo field in entityFields.GetOrAdd(entity.GetType(), GetEntityFields)) {
            try {
                object? simplified = Simplify(field.GetValue(entity));
                if (simplified is not null) result.TryAdd(field.Name, simplified);
            } catch { }
        }
        return result;
    }

    private static FieldInfo[] GetEntityFields(Type concreteType) {
        List<FieldInfo> result = [];
        for (Type? type = concreteType;
             type is not null && type != typeof(Entity) && type != typeof(object);
             type = type.BaseType) {
            foreach (FieldInfo field in type.GetFields(
                         BindingFlags.Instance
                         | BindingFlags.Public
                         | BindingFlags.NonPublic
                         | BindingFlags.DeclaredOnly)) {
                if (field.IsStatic) continue;
                result.Add(field);
                if (result.Count >= MaxReflectedFields) return [.. result];
            }
        }
        return [.. result];
    }

    private static float[]? ReadVector(Entity entity, params string[] names) {
        Type type = entity.GetType();
        foreach (string name in names) {
            try {
                PropertyInfo? property = type.GetProperty(
                    name,
                    BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic
                );
                if (property?.GetValue(entity) is Vector2 propertyVector) {
                    return [propertyVector.X, propertyVector.Y];
                }
                FieldInfo? field = type.GetField(
                    name,
                    BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic
                );
                if (field?.GetValue(entity) is Vector2 fieldVector) {
                    return [fieldVector.X, fieldVector.Y];
                }
            } catch { }
        }
        return null;
    }

    private static object? Simplify(object? value) {
        switch (value) {
            case null:
                return null;
            case bool or byte or sbyte or short or ushort or int or uint or long or ulong
                or float or double or string:
                return value;
            case Enum enumeration:
                return Convert.ToInt32(enumeration);
            case Vector2 vector:
                return new[] { vector.X, vector.Y };
            case Rectangle rectangle:
                return new[] { rectangle.X, rectangle.Y, rectangle.Width, rectangle.Height };
            case Array array when array.Length <= MaxCollectionItems:
                return SimplifyCollection(array);
            case IList list when list.Count <= MaxCollectionItems:
                return SimplifyCollection(list);
            default:
                return null;
        }
    }

    private static List<object?>? SimplifyCollection(IEnumerable collection) {
        List<object?> result = [];
        foreach (object? item in collection) {
            object? simplified = Simplify(item);
            if (simplified is null && item is not null) return null;
            result.Add(simplified);
        }
        return result;
    }
}
